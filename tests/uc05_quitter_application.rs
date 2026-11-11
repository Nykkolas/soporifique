//! Cartouche — spec exécutable UC-05 « Quitter l'application ».
//!
//! - Document source : `docs/usecases/uc05-quitter-application.md`
//!   (ce test joue le **scénario primaire** ; les extensions 2a et 5a
//!   auront leurs propres tests à l'implémentation).
//! - Coverage : instancié.
//! - Le test conduit le **vrai** runtime (`soporifique::runtime`) :
//!   model/update/view et décisions de fenêtre sont le code de
//!   production, pas une copie. Ne reste simulé que la partie OS :
//!   - étapes 1–2 : le menu de la barre d'état est rendu par l'hôte de
//!     la zone de notification (DBus/OS) — clics OS natifs non
//!     rejouables headless. Ses éléments sont vérifiés par inspection
//!     du modèle du menu (cf. `docs/RUST-EGUI.md`, `tray_menu`) : le
//!     test y active l'entrée « Quitter Soporifique » au point
//!     producteur de Messages, exactement comme le service tray ;
//!   - le miroir winit : notre `WindowEffect::Close` revient à la frame
//!     suivante sous forme d'évènement `Close` en entrée
//!     (indistinguable du ✕ de l'utilisateur), et sans `CancelClose`
//!     une fermeture demandée termine l'application ;
//!   - l'arrêt réel du processus : la vérité finale reste la recette
//!     manuelle.

use egui_kittest::{Harness, kittest::Queryable};
use soporifique::runtime::WindowEffect;

use common::Host;

mod common;

/// Activation de l'entrée « Quitter Soporifique » du menu de la barre
/// d'état : inspection du modèle du menu (cf. cartouche) puis jeu du
/// geste au point producteur de Messages. Échoue si l'entrée
/// disparaît, change de libellé ou émet le mauvais geste.
fn quit_entry_message() -> soporifique::message::Message {
    soporifique::tray_menu::menu()
        .into_iter()
        .find(|entry| entry.label == soporifique::labels::TRAY_MENU_QUIT)
        .unwrap_or_else(|| {
            panic!(
                "[étapes 1-2] le menu du clic droit doit contenir \
                 « {} » (libellé exact du glossaire — UC-05, étape 2)",
                soporifique::labels::TRAY_MENU_QUIT
            )
        })
        .message
}

// ------------------------------------------------------------------ le test

/// UC-05, scénario primaire : quitter depuis le menu de la barre d'état,
/// confirmation à l'écran, arrêt réel de l'application.
#[test]
fn quitting_from_tray_menu_shows_confirmation_then_exits() {
    // Précondition : l'application est lancée en mode barre d'état
    // système (host simulé pour la partie OS, cf. cartouche).
    let mut harness = Harness::new_ui_state(|ui, host: &mut Host| host.frame(ui), Host::new());

    // Étapes 1–2 : menu rendu par l'hôte de la barre d'état — ses
    // éléments sont vérifiés par inspection du modèle (cf. cartouche).
    // Étape 3 : l'utilisateur clique sur « Quitter Soporifique » — le
    // test active la vraie entrée du menu au point producteur.
    harness.state_mut().runtime.receive(quit_entry_message());
    harness.run();

    // Étape 4 : l'application affiche la demande de confirmation, avec
    // le texte exact du glossaire et les boutons « Oui » / « Non ».
    let confirmation = harness.query_by_label_contains(soporifique::labels::QUIT_CONFIRMATION_TEXT);
    assert!(
        confirmation.is_some(),
        "[étape 4] la demande de confirmation doit être affichée, texte exact du module labels"
    );
    let confirm_button = harness.query_by_label(soporifique::labels::QUIT_CONFIRM_BUTTON);
    assert!(
        confirm_button.is_some(),
        "[étape 4] le bouton « {} » doit être visible dans la demande de confirmation",
        soporifique::labels::QUIT_CONFIRM_BUTTON
    );

    // Étape 5 : l'utilisateur clique sur « Oui ».
    let button = harness.get_by_label(soporifique::labels::QUIT_CONFIRM_BUTTON);
    button.click();
    harness.run(); // le clic devient Message, update ordonne l'arrêt
    harness.run(); // le runtime traduit la commande en fermeture

    // Étape 6 : l'application ordonne sa fermeture au harnais.
    assert!(
        harness
            .state()
            .ordered_effects
            .contains(&WindowEffect::Close),
        "[étape 6] confirmer l'arrêt doit ordonner la fermeture au harnais"
    );

    // Étape 6 (suite) : la fermeture demandée revient en évènement
    // (miroir winit) — elle ne doit PAS être annulée par la garde
    // « ✕ ≡ Non », sinon l'application ne s'arrête jamais (bug de
    // recette UC-05, cf. docs/TODO.md) — et l'application s'arrête.
    harness.run();
    assert!(
        harness.state().exited,
        "[étape 6] cliquer « Oui » doit arrêter réellement l'application : \
         la fermeture ordonnée par elle-même ne doit pas être annulée"
    );
}

/// UC-05, extension 5a : « Non » referme la demande de confirmation,
/// l'application reste ouverte — jamais de fermeture ordonnée.
#[test]
fn cancelling_quit_closes_confirmation_and_keeps_the_app_running() {
    // Précondition : l'application est lancée en mode barre d'état
    // système ; le chemin OS jusqu'à la confirmation (étapes 1-4 du
    // scénario primaire) est couvert par le test du scénario primaire.
    let mut harness = Harness::new_ui_state(|ui, host: &mut Host| host.frame(ui), Host::new());

    harness.state_mut().runtime.receive(quit_entry_message());
    harness.run();
    assert!(
        harness
            .query_by_label_contains(soporifique::labels::QUIT_CONFIRMATION_TEXT)
            .is_some(),
        "[préparation] la demande de confirmation doit être affichée"
    );

    // 5a.1 : l'utilisateur clique sur « Non ».
    let cancel_button = harness.get_by_label(soporifique::labels::QUIT_CANCEL_BUTTON);
    cancel_button.click();
    harness.run();

    // 5a.2 : la demande de confirmation disparaît…
    assert!(
        harness
            .query_by_label_contains(soporifique::labels::QUIT_CONFIRMATION_TEXT)
            .is_none(),
        "[5a.2] la demande de confirmation doit disparaître après « Non »"
    );

    //     …et l'application reste ouverte : aucune fermeture ordonnée.
    let host = harness.state();
    assert!(
        !host.ordered_effects.contains(&WindowEffect::Close),
        "[5a.2] cliquer « Non » ne doit jamais ordonner la fermeture \
         (un bouton Non inversé avec Oui serait invisible sans ce test)"
    );
    assert!(
        !host.exited,
        "[5a.2] l'application doit rester ouverte après « Non »"
    );
}
