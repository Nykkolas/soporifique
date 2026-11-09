//! Cartouche — spec exécutable UC-05 « Quitter l'application ».
//!
//! - Document source : `docs/usecases/uc05-quitter-application.md`
//!   (ce test joue le **scénario primaire** ; les extensions 2a et 5a
//!   auront leurs propres tests à l'implémentation).
//! - Coverage : instancié.
//! - Non couvert (déclaré, cf. `docs/SPEC-UC.md` et `docs/RUST-EGUI.md`) :
//!   - étapes 1–2 : le menu de la barre d'état est rendu par l'hôte de la
//!     zone de notification (DBus/OS) — clics OS natifs non rejouables
//!     headless. Le test joue le rôle de l'hôte en activant l'entrée
//!     « Quitter Soporifique » au point d'entrée producteur de Messages,
//!     exactement comme le service tray le fera ;
//!   - étape 6 « disparaît de la barre d'état système » : le service tray
//!     est un détail d'infrastructure — l'arrêt est vérifié par la
//!     [`Command::ExitApplication`] que `update` ordonne au harnais ;
//!   - l'arrêt réel du processus : non observable dans un harnais de test.
//!     Le mini-harnais simule par contre en miroir simplifié ce que
//!     `main.rs` + eframe/winit font des Command et évènements de
//!     fermeture (cf. mini-harnais ci-dessous) — c'est ce miroir qui
//!     reproduit le bug de recette « Oui ne quitte pas » ; la vérité
//!     finale reste la recette manuelle.

use egui_kittest::{kittest::Queryable, Harness};
use soporifique::{command::Command, labels, message::Message, model::Model, update::update, view::view};

// ------------------------------------------------------------------ harnais

/// Mini-harnais TEA de la spec (interpréteur de pas, cf. `docs/SPEC-UC.md`) :
/// modèle + commandes exécutées par le harnais, une frame = vue + update.
///
/// Il simule aussi ce que le harnais réel (`main.rs` + eframe/winit) fait
/// des Command et évènements de fenêtre, à l'écran près : winit renvoie
/// un `ViewportCommand::Close` sous forme d'évènement `Close` lisible en
/// entrée à la frame suivante, indistinguable du ✕ de l'utilisateur.
/// Le drapeau `self_close` (miroir du `main.rs`) départage les deux :
/// laisser passer notre propre fermeture (UC-05 étape 6), ne jamais
/// laisser passer celle de l'utilisateur (UC-04 : ✕ ≡ « Non »).
struct App {
    model: Model,
    /// Commandes ordonnées par `update`, en attente d'exécution par le
    /// harnais (jamais exécutées dans `update`).
    pending_commands: Vec<Command>,
    executed_commands: Vec<Command>,
    /// Fermeture ordonnée par l'application elle-même (étape 6).
    self_close: bool,
    /// Évènement `Close` de fenêtre en attente pour la frame suivante
    /// (miroir winit : ✕ utilisateur OU notre propre commande Close).
    pending_close_event: bool,
    /// Arrêt réel de l'application (la fermeture est passée).
    app_exited: bool,
}

impl App {
    fn new() -> Self {
        Self {
            model: Model::default(),
            pending_commands: Vec::new(),
            executed_commands: Vec::new(),
            self_close: false,
            pending_close_event: false,
            app_exited: false,
        }
    }

    /// Reçoit un Message comme n'importe quel producteur en émettrait
    /// (tray, UI, timer) : `update` pur, commandes mises en attente
    /// d'exécution par le harnais.
    fn receive(&mut self, message: Message) {
        let (model, commands) = update(self.model.clone(), message);
        self.model = model;
        self.pending_commands.extend(commands);
    }

    /// La partie du harnais réel qui vit dans `App::logic` (main.rs) :
    /// évènements de fenêtre puis exécution des Command.
    fn step_logic(&mut self) {
        if self.pending_close_event {
            self.pending_close_event = false;
            if self.self_close {
                // Fermeture ordonnée par l'application (UC-05,
                // étape 6) : laisser passer — l'application s'arrête.
                self.app_exited = true;
            } else {
                // ✕ de l'utilisateur ≡ « Non » (règle UC-04, extension
                // 5a de UC-05) : annuler la fermeture, ne jamais quitter.
                self.receive(Message::QuitCancelled);
            }
        }

        // Exécution des Command ordonnées par `update`.
        let commands = std::mem::take(&mut self.pending_commands);
        for command in commands {
            self.executed_commands.push(command.clone());
            match command {
                Command::ExitApplication => {
                    // Miroir main.rs : drapeau self_close puis
                    // ViewportCommand::Close — que winit renverra comme
                    // évènement Close à la frame suivante.
                    self.self_close = true;
                    self.pending_close_event = true;
                }
            }
        }
    }

    /// Une frame : logique du harnais puis vue rend le modèle et émet
    /// ses Messages (même ordre que main.rs : logic puis ui).
    fn render(&mut self, ui: &mut egui::Ui) {
        self.step_logic();
        let mut outbox = Vec::new();
        view(&self.model, ui, &mut outbox);
        for message in outbox {
            self.receive(message);
        }
    }
}

// -------------------------------------------------------------- le test

/// UC-05, scénario primaire : quitter depuis le menu de la barre d'état,
/// confirmation à l'écran, arrêt ordonné au harnais.
#[test]
fn quitting_from_tray_menu_shows_confirmation_then_exits() {
    // Précondition : l'application est lancée en mode barre d'état système
    // (simulée par le harnais — cf. cartouche pour ce qui n'est pas rejouable).
    let mut harness = Harness::new_ui_state(|ui, app: &mut App| app.render(ui), App::new());

    // Étapes 1–2 : menu rendu par l'hôte de la barre d'état — non couvert.
    // Étape 3 : l'utilisateur clique sur « Quitter Soporifique » — le test
    // joue le rôle de l'hôte de la zone de notification (cartouche).
    harness.state_mut().receive(Message::QuitRequested);
    harness.run();

    // Étape 4 : l'application affiche la demande de confirmation, avec
    // le texte exact du glossaire et les boutons « Oui » / « Non ».
    let confirmation = harness.query_by_label_contains(labels::QUIT_CONFIRMATION_TEXT);
    assert!(
        confirmation.is_some(),
        "[étape 4] la demande de confirmation doit être affichée, texte exact du module labels"
    );
    let confirm_button = harness.query_by_label(labels::QUIT_CONFIRM_BUTTON);
    assert!(
        confirm_button.is_some(),
        "[étape 4] le bouton « {} » doit être visible dans la demande de confirmation",
        labels::QUIT_CONFIRM_BUTTON
    );

    // Étape 5 : l'utilisateur clique sur « Oui ».
    let button = harness.get_by_label(labels::QUIT_CONFIRM_BUTTON);
    button.click();
    harness.run();

    // Frame suivante : le harnais exécute la commande d'arrêt.
    harness.run();

    // Étape 6 : l'application ordonne son arrêt au harnais.
    let exited_commanded = harness
        .state()
        .executed_commands
        .contains(&Command::ExitApplication);
    assert!(
        exited_commanded,
        "[étape 6] confirmer l'arrêt doit ordonner l'arrêt de l'application au harnais"
    );

    // Étape 6 — reproduction du bug de recette (« Oui ne quitte pas ») :
    // winit renvoie le Close que NOUS avons demandé sous forme d'évènement
    // `Close` en entrée ; le harnais actuel l'annule via la garde ✕ ≡ « Non »
    // (miroir du main.rs en l'état, cf. mini-harnais ci-dessus et
    // docs/TODO.md). L'application doit pourtant s'arrêter : attendue
    // ROUGE aujourd'hui — c'est le bug, pas une régression du test.
    harness.run();
    assert!(
        harness.state().app_exited,
        "[étape 6] cliquer « Oui » doit arrêter réellement l'application : \
         le garde-fou ✕ ≡ « Non » annule aujourd'hui la fermeture que \
         l'application s'est ordonnée à elle-même"
    );
}
