//! Cartouche — spec exécutable UC-04 « Accéder à la configuration
//! actuelle ».
//!
//! - Document source : `docs/usecases/uc04-acceder-configuration.md`
//!   (ce fichier joue le **scénario primaire** et les **extensions
//!   1a, 2a et 2b** ; l'extension 4a aura son propre test à
//!   l'implémentation).
//! - Coverage : instancié.
//! - Périmètre : la gestion de la sauvegarde vit dans ce UC (étape 5 :
//!   persister sur disque à « Enregistrer », relecture au lancement).
//!   Le paramètre affiché est un **paramètre d'essai** (case à cocher),
//!   seul but permettre à l'utilisateur de tester la chaîne lui-même —
//!   les paramètres réels viendront avec leurs UC (le seuil avec
//!   l'UC-01), la mécanique de sauvegarde restera ici.
//! - Le test conduit le **vrai** runtime (`soporifique::runtime`) :
//!   model/update/view et décisions sont le code de production, pas une
//!   copie. Ne reste simulé que la partie OS :
//!   - étape 1 / 1a.1-2 : le clic gauche (signal `Activate`) et le menu
//!     du clic droit arrivent par DBus/StatusNotifierItem — non
//!     rejouables headless. Les gestes sont joués au point producteur
//!     de Messages, et les éléments du menu sont vérifiables **par
//!     inspection du modèle du menu** (cf. `docs/RUST-EGUI.md`) : le
//!     modèle vit dans la lib, le service DBus (`tray.rs`) n'est qu'un
//!     détail d'infrastructure ;
//!   - la fenêtre racine elle-même (montrer/cacher une vraie fenêtre
//!     Cinnamon) : le test vérifie les **ordres** (`Show`/`Hide`)
//!     confiés au harnais, pas la fenêtre réelle — recette manuelle ;
//!   - l'écriture disque est **réelle** (fichier temporaire du test) :
//!     le test vérifie la présence du fichier, son **format** et les
//!     **paramètres** qui y sont enregistrés, puis **relance
//!     l'application** (second lancement) pour vérifier la relecture
//!     des paramètres au démarrage ;
//!   - le fichier réel de l'utilisateur (`$XDG_CONFIG_HOME/...`) est
//!     de la colle du harnais (`main.rs`) — non rejouable headless ;
//!     le chemin injecté par le test est un fichier temporaire.
//!   - 1a.4 « l'application ferme le menu » : le menu est dessiné par
//!     l'hôte de la barre d'état, c'est lui qui le referme à
//!     l'activation d'une entrée — recette manuelle, aucun ordre à
//!     vérifier de notre côté.
//! - Le format exact du fichier (une ligne `clé=valeur` par
//!   paramètre) est un choix d'implémentation figé ici exprès — à
//!   discrétion de relecture ci-dessous.

use std::path::PathBuf;

use egui_kittest::Harness;
use egui_kittest::kittest::{NodeT, Queryable};
use soporifique::message::Message;
use soporifique::runtime::WindowEffect;
use soporifique::store::SavedConfig;

use common::Host;

mod common;

// -------------------------------------------------- garde d'anticipation

/// Rôles AccessKit des widgets de saisie : un paramètre peut être
/// n'importe quel widget de saisie (case à cocher, champ texte,
/// slider, options…).
const INPUT_ROLES: &[egui::Role] = &[
    egui::Role::Slider,
    egui::Role::TextInput,
    egui::Role::MultilineTextInput,
    egui::Role::CheckBox,
    egui::Role::RadioButton,
    egui::Role::Switch,
    egui::Role::ComboBox,
    egui::Role::EditableComboBox,
    egui::Role::SpinButton,
];

/// Libellés des paramètres spécifiés par les UC — aujourd'hui le seul
/// paramètre d'essai (cf. cartouche). Contrat : la fenêtre de
/// configuration n'affiche QUE ces paramètres. À mettre à jour à
/// chaque paramètre validé en UC : un seul point à retoucher, jamais
/// les UC précédentes.
const CONFIG_PARAMS: &[&str] = &[soporifique::labels::TEST_PARAM_LABEL];

// ------------------------------------------------------------------ le test

/// UC-04, scénario primaire : clic gauche sur l'icône, la fenêtre de
/// configuration s'ouvre et montre le paramètre, l'utilisateur le
/// modifie, « Enregistrer » persiste les paramètres sur disque (format
/// et contenu vérifiés) et referme la fenêtre — et un second
/// lancement de l'application relit ces paramètres au démarrage.
#[test]
fn left_click_opens_config_then_save_persists_to_disk_and_next_launch_reloads() {
    let path = temp_config_path("primary");
    clean_config_file(&path);

    // Précondition : l'application est lancée en mode barre d'état
    // système (host simulé pour la partie OS, cf. cartouche) — premier
    // lancement, sans fichier de configuration sur le disque.
    let mut harness = Host::booted_harness(&path);

    // Étape 1 : l'utilisateur clique gauche sur l'icône — le test joue
    // le rôle de l'hôte de la zone de notification (signal `Activate`,
    // cf. cartouche) ; le tray réel émettra ce même Message.
    harness
        .state_mut()
        .runtime
        .receive(Message::ConfigOpenRequested);
    harness.run();

    // Étape 2 : l'application ouvre la fenêtre — l'ordre de montrer la
    // fenêtre part vers le harnais, et sa passe de rendu suit.
    let show_config = WindowEffect::Show {
        title: soporifique::labels::CONFIG_WINDOW_TITLE.into(),
    };
    assert!(
        harness.state().ordered_effects.contains(&show_config),
        "[étape 2] ouvrir la configuration doit ordonner d'afficher la fenêtre"
    );
    assert!(
        harness
            .state()
            .ordered_effects
            .contains(&WindowEffect::Repaint),
        "[étape 2] une fenêtre cachée ne se réveille pas sans repaint"
    );

    //     La fenêtre affiche le paramètre d'essai, décoché par défaut
    // (premier lancement : aucun paramètre enregistré sur le disque).
    assert!(
        harness
            .query_all_by_role_and_label(
                egui::Role::CheckBox,
                soporifique::labels::TEST_PARAM_LABEL
            )
            .next()
            .is_some(),
        "[étape 2] le paramètre d'essai doit être affiché, en case à cocher étiquetée du glossaire"
    );
    assert_eq!(
        checkbox_toggled(&harness),
        Some(egui::accesskit::Toggled::False),
        "[étape 2] sans paramètre enregistré, la case est décochée"
    );

    //     Garde d'anticipation : la fenêtre n'affiche QUE les
    // paramètres spécifiés par les UC. La garde énumère tous les
    // widgets de saisie, quel que soit leur type (cf. INPUT_ROLES),
    // et vérifie que leurs libellés forment exactement les paramètres
    // déclarés.
    let inputs: Vec<Option<String>> = INPUT_ROLES
        .iter()
        .flat_map(|role| harness.query_all_by_role(*role))
        .map(|node| node.accesskit_node().label())
        .collect();
    let mut found_labels: Vec<String> = inputs.into_iter().map(|l| l.unwrap_or_default()).collect();
    found_labels.sort();
    found_labels.dedup();
    let expected_labels: Vec<String> = CONFIG_PARAMS.iter().map(|l| l.to_string()).collect();
    assert_eq!(
        found_labels, expected_labels,
        "[étape 2] la fenêtre de configuration ne doit afficher que les \
         paramètres spécifiés : un widget de saisie en surnombre, au \
         libellé inconnu ou absent, révèle un paramètre jamais spécifié \
         par les UC"
    );

    // Étape 3 : l'utilisateur modifie le paramètre (coche la case).
    let checkbox =
        harness.get_by_role_and_label(egui::Role::CheckBox, soporifique::labels::TEST_PARAM_LABEL);
    checkbox.click();
    harness.run();
    assert_eq!(
        checkbox_toggled(&harness),
        Some(egui::accesskit::Toggled::True),
        "[étape 3] cocher la case doit changer le paramètre affiché"
    );
    assert!(
        harness.state().runtime.model().test_param,
        "[étape 3] le paramètre modifié vit dans le modèle"
    );

    // Étape 4 : l'utilisateur clique sur « Enregistrer ».
    let save_button = harness.get_by_label(soporifique::labels::SAVE_BUTTON);
    save_button.click();
    harness.run(); // le clic devient Message : la sauvegarde est demandée
    harness.run(); // la commande devient effet ; l'hôte exécute le disque
    harness.run(); // le résultat revient en Message : la fenêtre se referme

    // Étape 5 : l'application sauvegarde les paramètres — l'ordre part
    // vers le harnais avec les valeurs au moment du clic, et le disque
    // (vérité extérieure, cf. cartouche) porte le résultat :
    // le fichier existe, son format est celui de la spec (une ligne
    // `clé=valeur` par paramètre) et les paramètres enregistrés y
    // sont exacts.
    let save_effect = WindowEffect::SaveConfig {
        saved: SavedConfig { test_param: true },
    };
    assert!(
        harness.state().ordered_effects.contains(&save_effect),
        "[étape 5] « Enregistrer » doit ordonner de persister les paramètres \
         du moment du clic"
    );
    let contents = std::fs::read_to_string(&path)
        .expect("[étape 5] le fichier de configuration doit exister sur le disque");
    assert_eq!(
        contents, "test_param=true\n",
        "[étape 5] le format du fichier (une ligne clé=valeur) et le \
         paramètre enregistré doivent être exactement ceux de la spec"
    );

    // Étape 6 : la fenêtre de configuration disparaît — la refermeture
    // doit être ordonnée APRÈS la sauvegarde (le hide anti-flash du
    // démarrage ne doit jamais satisfaire cette étape). « Titre à
    // remettre » est un détail interne du harnais (titre de repos de
    // la fenêtre cachée).
    let save_position = harness
        .state()
        .ordered_effects
        .iter()
        .position(|effect| *effect == save_effect)
        .expect("[étape 6] l'effet de sauvegarde doit avoir été ordonné");
    assert!(
        harness.state().ordered_effects[save_position..]
            .iter()
            .any(|effect| matches!(effect, WindowEffect::Hide { .. })),
        "[étape 6] « Enregistrer » doit ordonner de refermer la fenêtre de configuration"
    );
    assert!(
        harness
            .query_all_by_role_and_label(
                egui::Role::CheckBox,
                soporifique::labels::TEST_PARAM_LABEL
            )
            .next()
            .is_none(),
        "[étape 6] le paramètre ne doit plus être affiché : la fenêtre est refermée"
    );

    // Success guarantee : l'application continue à tourner dans la
    // barre d'état système — aucune fermeture n'a jamais été ordonnée.
    assert!(
        !harness
            .state()
            .ordered_effects
            .contains(&WindowEffect::Close),
        "la configuration ne doit jamais arrêter l'application (barre d'état)"
    );
    assert!(!harness.state().exited, "l'application doit rester lancée");

    //     Relecture au lancement de l'application : un second
    // lancement boote depuis le même disque et retrouve le paramètre
    // sauvé — sans jamais cliquer « Enregistrer » cette fois. La
    // vérité lue est celle affichée (l'état coché), pas un recalcul.
    let mut second_launch = Host::booted_harness(&path);
    second_launch
        .state_mut()
        .runtime
        .receive(Message::ConfigOpenRequested);
    second_launch.run();
    assert_eq!(
        checkbox_toggled(&second_launch),
        Some(egui::accesskit::Toggled::True),
        "[relancement] l'application doit relire le fichier de configuration \
         au démarrage : le paramètre sauvé au lancement précédent doit \
         s'afficher tel quel"
    );

    // Part propre : le fichier temporaire du test ne doit rien à la
    // prochaine exécution.
    clean_config_file(&path);
}

/// UC-04, extension 1a : le clic droit sur l'icône ouvre le menu de la
/// barre d'état, dont l'option « Configurer... » affiche la fenêtre de
/// configuration du scénario primaire.
#[test]
fn right_click_menu_configure_entry_opens_the_config_window() {
    // 1a.1-2 : l'utilisateur clique droit, l'hôte de la barre d'état
    // (Cinnamon/DBus) dessine le menu depuis notre modèle — les
    // éléments du menu sont vérifiés par inspection du modèle du menu
    // (cf. `docs/RUST-EGUI.md` et cartouche) : le test y cherche
    // l'option « Configurer... ».
    let configure = soporifique::tray_menu::menu()
        .into_iter()
        .find(|entry| entry.label == soporifique::labels::TRAY_MENU_CONFIGURE)
        .unwrap_or_else(|| {
            panic!(
                "[1a.2] le menu du clic droit doit contenir une option « {} » \
                 (configurer = terme du glossaire : libellé exact)",
                soporifique::labels::TRAY_MENU_CONFIGURE
            )
        });

    // 1a.3 : l'utilisateur clique sur cette option — le test joue
    // l'hôte de la zone de notification au même point producteur que le
    // service tray réel : l'activation de l'entrée émet son Message.
    let path = temp_config_path("ext-1a");
    clean_config_file(&path);
    let mut harness = Host::booted_harness(&path);
    harness.state_mut().runtime.receive(configure.message);
    harness.run();

    //     …et la fenêtre du scénario primaire s'affiche (étape 2 du
    // primaire), paramètre à l'écran.
    assert!(
        harness
            .query_all_by_role_and_label(
                egui::Role::CheckBox,
                soporifique::labels::TEST_PARAM_LABEL
            )
            .next()
            .is_some(),
        "[1a.3] activer « Configurer... » doit afficher la fenêtre du \
         scénario primaire, paramètres visibles"
    );
    assert!(
        !harness
            .state()
            .ordered_effects
            .contains(&WindowEffect::Close),
        "[1a.3] activer « Configurer... » ne doit jamais ordonner l'arrêt de \
         l'application (l'application vit dans la barre d'état)"
    );

    // 1a.4 : la refermeture du menu avant la fenêtre est la vérité de
    // l'hôte OS (Cinnamon referme son menu à l'activation) — recette
    // manuelle, cf. cartouche.

    clean_config_file(&path);
}

/// UC-04, extension 2a : la fenêtre de configuration est déjà
/// ouverte, mais en arrière-plan ; un nouveau clic gauche sur l'icône
/// la ramène à l'avant-plan, sans jamais la refermer ni la rouvrir.
#[test]
fn second_left_click_raises_the_background_config_window() {
    // Précondition de l'extension : la fenêtre de configuration est
    // affichée (scénario primaire, étapes 1-2). « En arrière-plan »
    // est une vérité du gestionnaire de fenêtres — la scène headless
    // se joue à l'identique, la vraie remise en avant-plan est
    // recette manuelle.
    let path = temp_config_path("ext-2a");
    clean_config_file(&path);
    let mut harness = Host::booted_harness(&path);
    harness
        .state_mut()
        .runtime
        .receive(Message::ConfigOpenRequested);
    harness.run();
    assert!(
        harness
            .query_all_by_role_and_label(
                egui::Role::CheckBox,
                soporifique::labels::TEST_PARAM_LABEL
            )
            .next()
            .is_some(),
        "[précondition 2a] la fenêtre de configuration doit être affichée \
         avant le geste de l'extension"
    );
    let show_position = harness
        .state()
        .ordered_effects
        .iter()
        .position(|effect| matches!(effect, WindowEffect::Show { .. }))
        .expect(
            "[précondition 2a] l'affichage de la fenêtre doit avoir été \
         ordonné",
        );

    // Vérité inversée : la première ouverture n'ordonne aucun
    // avant-plan — le geste 2a ne ramène à l'avant-plan que ce qui est
    // déjà à l'écran (remonter ce qui s'ouvre serait un bruit de focus).
    assert!(
        !harness
            .state()
            .ordered_effects
            .contains(&WindowEffect::Raise),
        "[précondition 2a] ouvrir la fenêtre ne doit pas ordonner \
         l'avant-plan : seule la ramener le fait"
    );

    // 2a : nouveau clic gauche — le test joue l'hôte de la zone de
    // notification au point producteur, comme l'étape 1 du primaire ;
    // le tray réel émettra ce même Message.
    harness
        .state_mut()
        .runtime
        .receive(Message::ConfigOpenRequested);
    harness.run();

    // L'application ramène la fenêtre à l'avant-plan : l'ordre part
    // vers le harnais (exécuté par winit — focus, non rejouable
    // headless, cf. cartouche).
    assert!(
        harness.state().ordered_effects[show_position..].contains(&WindowEffect::Raise),
        "[2a] un clic gauche alors que la configuration est déjà affichée \
         doit ordonner la remise en avant-plan de la fenêtre"
    );

    // La fenêtre n'est ni refermée ni cachée pour être ramenée : un
    // cache-remonté serait un clignotement, pas un avant-plan.
    assert!(
        !harness.state().ordered_effects[show_position..]
            .iter()
            .any(|effect| matches!(effect, WindowEffect::Hide { .. })),
        "[2a] ramener la fenêtre à l'avant-plan ne doit jamais passer par \
         un cache-remonté de la fenêtre"
    );
    assert!(
        harness
            .query_all_by_role_and_label(
                egui::Role::CheckBox,
                soporifique::labels::TEST_PARAM_LABEL
            )
            .next()
            .is_some(),
        "[2a] la fenêtre reste affichée, paramètre visible"
    );

    // Success guarantee : l'application continue à tourner dans la
    // barre d'état système — aucune fermeture n'a été ordonnée.
    assert!(
        !harness
            .state()
            .ordered_effects
            .contains(&WindowEffect::Close),
        "[2a] ramener la fenêtre à l'avant-plan ne doit jamais arrêter \
         l'application (barre d'état)"
    );
    assert!(!harness.state().exited, "l'application doit rester lancée");

    clean_config_file(&path);
}

/// UC-04, extension 2b : la fenêtre de configuration est déjà
/// ouverte (en arrière-plan) ; l'utilisateur passe par le menu du
/// clic droit et son option « Configurer... » — la fenêtre revient à
/// l'avant-plan, sans jamais être refermée ni rouverte.
#[test]
fn configure_menu_entry_raises_the_open_config_window() {
    // 2b.1 : le menu du clic droit est dessiné par l'hôte de la barre
    // d'état (Cinnamon/DBus) — vérité OS, recette manuelle ; le test
    // inspecte le modèle du menu (même mécanique que l'extension 1a,
    // cf. cartouche) et y cherche l'option « Configurer... ».
    let configure = soporifique::tray_menu::menu()
        .into_iter()
        .find(|entry| entry.label == soporifique::labels::TRAY_MENU_CONFIGURE)
        .unwrap_or_else(|| {
            panic!(
                "[2b.1] le menu du clic droit doit contenir une option « {} » \
                 (configurer = terme du glossaire : libellé exact)",
                soporifique::labels::TRAY_MENU_CONFIGURE
            )
        });

    // Précondition de l'extension : la fenêtre de configuration est
    // affichée (scénario primaire, étapes 1-2). « En arrière-plan »
    // est une vérité du gestionnaire de fenêtres — recette manuelle,
    // déjà confirmée en réel.
    let path = temp_config_path("ext-2b");
    clean_config_file(&path);
    let mut harness = Host::booted_harness(&path);
    harness
        .state_mut()
        .runtime
        .receive(Message::ConfigOpenRequested);
    harness.run();
    assert!(
        harness
            .query_all_by_role_and_label(
                egui::Role::CheckBox,
                soporifique::labels::TEST_PARAM_LABEL
            )
            .next()
            .is_some(),
        "[précondition 2b] la fenêtre de configuration doit être affichée \
         avant le geste de l'extension"
    );
    let show_position = harness
        .state()
        .ordered_effects
        .iter()
        .position(|effect| matches!(effect, WindowEffect::Show { .. }))
        .expect(
            "[précondition 2b] l'affichage de la fenêtre doit avoir été \
         ordonné",
        );
    assert!(
        !harness
            .state()
            .ordered_effects
            .contains(&WindowEffect::Raise),
        "[précondition 2b] ouvrir la fenêtre ne doit pas ordonner \
         l'avant-plan : seule la ramener le fait"
    );

    // 2b.2-3 : l'utilisateur active « Configurer... » — le test joue
    // l'hôte de la zone de notification au même point producteur que
    // le service tray réel : l'entrée du modèle émet son Message.
    harness.state_mut().runtime.receive(configure.message);
    harness.run();

    // La fenêtre revient à l'avant-plan : l'ordre part vers le
    // harnais (exécuté par winit — focus, non rejouable headless,
    // cf. cartouche).
    assert!(
        harness.state().ordered_effects[show_position..].contains(&WindowEffect::Raise),
        "[2b.3] activer « Configurer... » pendant que la fenêtre est déjà \
         affichée doit ordonner la remise en avant-plan"
    );

    // La fenêtre n'est ni refermée ni cachée pour être ramenée : un
    // cache-remonté serait un clignotement, pas un avant-plan.
    assert!(
        !harness.state().ordered_effects[show_position..]
            .iter()
            .any(|effect| matches!(effect, WindowEffect::Hide { .. })),
        "[2b.3] ramener la fenêtre à l'avant-plan ne doit jamais passer par \
         un cache-remonté de la fenêtre"
    );
    assert!(
        harness
            .query_all_by_role_and_label(
                egui::Role::CheckBox,
                soporifique::labels::TEST_PARAM_LABEL
            )
            .next()
            .is_some(),
        "[2b.3] la fenêtre reste affichée, paramètre visible"
    );

    // Success guarantee : l'application continue à tourner dans la
    // barre d'état système — aucune fermeture n'a été ordonnée.
    assert!(
        !harness
            .state()
            .ordered_effects
            .contains(&WindowEffect::Close),
        "[2b.3] ramener la fenêtre à l'avant-plan ne doit jamais arrêter \
         l'application (barre d'état)"
    );
    assert!(!harness.state().exited, "l'application doit rester lancée");

    clean_config_file(&path);
}

/// État coché du paramètre d'essai à l'écran (vérité affichée, pas
/// recalculée du modèle).
fn checkbox_toggled(harness: &Harness<'_, Host>) -> Option<egui::accesskit::Toggled> {
    let checkbox =
        harness.get_by_role_and_label(egui::Role::CheckBox, soporifique::labels::TEST_PARAM_LABEL);
    checkbox.accesskit_node().toggled()
}

/// Fichier de configuration temporaire unique pour ce test : chemin
/// temporaire au nom du test + pid (les fichiers du projet ne sont
/// jamais touchés).
fn temp_config_path(name: &str) -> PathBuf {
    std::env::temp_dir().join(format!(
        "soporifique-uc04-{name}-{}.config",
        std::process::id()
    ))
}

/// Part propre avant/après : la vérité extérieure du test (le disque)
/// ne doit rien au lancement précédent.
fn clean_config_file(path: &PathBuf) {
    if path.exists() {
        std::fs::remove_file(path).expect("nettoyage du fichier temporaire");
    }
}
