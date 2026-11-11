//! Logique du *shell* (hôte graphique) : les décisions de la fenêtre
//! unique de l'application, pures et testables.
//!
//! Les effets eux-mêmes sont exécutés par le harnais seul (`main.rs`),
//! qui traduit les [`WindowEffect`] retournés en commandes egui/winit
//! (cf. `docs/RUST-TEA.md` : le harnais n'exécute que des effets —
//! décider est de la logique, donc hors du binaire pour rester sous
//! tests ; cf. Learn Rust ch. 13.5 : « main.rs fin, la logique dans la
//! lib, car seule la lib est testable »).

use crate::{
    command::Command, labels, message::Message, model::Model, store::SavedConfig, update::update,
    view::view,
};

/// Tick de `step` à partir duquel la fenêtre racine peut être cachée :
/// eframe la force visible après son premier rendu (anti-flash blanc,
/// cf. le bloc 2 de `step`) — cacher avant serait déjoué par ce forçage
/// et laisserait la fenêtre à l'écran. Constante couplée au comportement
/// d'eframe, à revoir à chaque upgrade.
const ROOT_HIDE_TICK: u32 = 2;

/// Effet ordonné au harnais — jamais exécuté ici. La fenêtre vit à
/// l'écran dans la plupart des cas, mais tout ce que le harnais doit
/// exécuter passe par ici (aujourd'hui aussi : la persistance des
/// paramètres, UC-04 étape 5) : un seul chemin commande → effet.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum WindowEffect {
    /// Montrer la fenêtre racine pour porter une demande à l'utilisateur.
    Show {
        /// Titre de la fenêtre tant qu'elle est montrée.
        title: String,
    },
    /// Cacher la fenêtre racine (titre remis).
    Hide {
        /// Titre à remettre.
        title: String,
    },
    /// Persister les paramètres sur le disque (UC-04, étape 5) — le
    /// harnais exécute son stockage injecté puis rapporte le résultat
    /// (`ConfigSaved` / `ConfigSaveFailed`).
    SaveConfig {
        /// Paramètres au moment du clic « Enregistrer ».
        saved: SavedConfig,
    },
    /// Annuler une fermeture demandée (garde « ✕ annule sans
    /// conséquence » : fermer ne quitte jamais l'application de la
    /// main de l'utilisateur — référence UC-04, extension 5a de UC-05).
    CancelClose,
    /// Fermer l'application : fermeture ordonnée par elle-même (UC-05,
    /// étape 6). Le harnais la marque comme la nôtre AVANT qu'elle ne
    /// revienne en évènement (winit réfléchit tout Close en entrée).
    Close,
    /// Redemander une passe (une fenêtre cachée ne se réveille pas
    /// seule, cf. `docs/RUST-EGUI.md` — repaint).
    Repaint,
}

/// Le runtime : détient le modèle et les états du shell, prend les
/// décisions de fenêtre, délègue à `update` (pur) et rend la vue.
/// Aucune connaissance de egui-chat/winit : la vérité extérieure entre
/// par paramètres (`close_requested`), sort en effets.
pub struct Runtime {
    model: Model,
    /// Command ordonnées par `update`, en attente d'exécution.
    pending_commands: Vec<Command>,
    /// Ticks de `step` — cf. la cachage initial d'eframe.
    logic_ticks: u32,
    /// Le premier re-cachage d'eframe est-il fait ?
    root_hidden_once: bool,
    /// La fenêtre porte-t-elle une demande à l'écran ?
    window_shown: bool,
    /// Fermeture ordonnée par l'application elle-même (UC-05, étape 6) :
    /// winit renvoie notre propre fermeture sous forme d'évènement
    /// `Close` en entrée, indistinguable du ✕ de l'utilisateur — ce
    /// drapeau les départage (bug de recette UC-05, cf. docs/TODO.md).
    self_close: bool,
}

impl Default for Runtime {
    fn default() -> Self {
        Self::new(Model::default())
    }
}

impl Runtime {
    /// Le modèle initial vient du lancement : paramètres relus du
    /// disque par le harnais via `store::boot_model` (UC-04).
    pub fn new(model: Model) -> Self {
        Self {
            model,
            pending_commands: Vec::new(),
            logic_ticks: 0,
            root_hidden_once: false,
            window_shown: false,
            self_close: false,
        }
    }

    /// Le modèle courant (lecture seule — il vit ici, ne vit qu'ici).
    pub fn model(&self) -> &Model {
        &self.model
    }

    /// Un producteur a émis (tray, vue, timer) : `update` pur, les
    /// Command attendent l'exécution par le harnais.
    pub fn receive(&mut self, message: Message) {
        let (model, commands) = update(self.model.clone(), message);
        self.model = model;
        self.pending_commands.extend(commands);
    }

    /// Un cycle côté hôte : décisions pures, effets retournés au harnais.
    ///
    /// `close_requested` : un évènement `Close` de fenêtre est présent
    /// en entrée ce cycle — ✕ de l'utilisateur OU notre propre fermeture
    /// réfléchie par winit (indistinguables ; le drapeau `self_close`
    /// tranche).
    pub fn step(&mut self, close_requested: bool) -> Vec<WindowEffect> {
        let mut effects = Vec::new();

        // 1. Garde « ✕ annule sans conséquence » : la fermeture demandée
        //    par l'utilisateur est annulée — sauf celle que nous avons
        //    ordonnée. Le ✕ referme chaque demande ouverte sans rien
        //    ordonner ni enregistrer : « Non » pour la confirmation
        //    (UC-05), « Annuler » pour la configuration (UC-04 — champ
        //    non tranché par l'UC, défaut d'implémentation cohérent).
        if close_requested && !self.self_close {
            effects.push(WindowEffect::CancelClose);
            self.receive(Message::QuitCancelled);
            self.receive(Message::ConfigCancelled);
        }

        // 2. eframe force l'affichage de la fenêtre après le premier
        //    rendu (anti-flash blanc,
        //    `eframe::native::epi_integration::post_rendering`) : la
        //    cacher seulement une fois ce rendu passé. Cacher plus tôt
        //    serait déjoué par l'affichage forcé du premier rendu ;
        //    ce seuil est couplé au comportement d'eframe (constante
        //    ci-dessous) — à revoir à chaque upgrade d'eframe.
        self.logic_ticks = self.logic_ticks.saturating_add(1);
        if self.logic_ticks >= ROOT_HIDE_TICK && !self.root_hidden_once {
            self.root_hidden_once = true;
            effects.push(WindowEffect::Hide {
                title: labels::APP_TITLE.into(),
            });
        }

        // 3. La fenêtre ne vit à l'écran que pour porter une demande :
        //    la confirmation d'arrêt (UC-05) ou la fenêtre de
        //    configuration (UC-04). `Repaint` porte la passe qui suit :
        //    une fenêtre cachée ne se réveille pas toute seule. La
        //    configuration prime sur le titre (elle est la demande
        //    principale quand les deux coexistent).
        let demand_title = if self.model.config_window.is_some() {
            Some(labels::CONFIG_WINDOW_TITLE)
        } else if self.model.quit_request.is_some() {
            Some(labels::TRAY_MENU_QUIT)
        } else {
            None
        };
        match (demand_title, self.window_shown) {
            (Some(title), false) => {
                self.window_shown = true;
                effects.push(WindowEffect::Show {
                    title: title.into(),
                });
                effects.push(WindowEffect::Repaint);
            }
            (None, true) => {
                self.window_shown = false;
                effects.push(WindowEffect::Hide {
                    title: labels::APP_TITLE.into(),
                });
            }
            _ => {}
        }

        // 4. Les Command ordonnées par `update` deviennent des effets —
        //    jamais exécutées dans `update`, ni ici.
        for command in std::mem::take(&mut self.pending_commands) {
            match command {
                Command::ExitApplication => {
                    self.self_close = true;
                    effects.push(WindowEffect::Close);
                }
                Command::SaveConfig { saved } => {
                    effects.push(WindowEffect::SaveConfig { saved });
                }
            }
        }

        effects
    }

    /// Rendu de la vue : le harnais fournit l'Ui egui ; la vue lit le
    /// modèle, émet ses Messages, `update` les digère.
    pub fn render(&mut self, ui: &mut egui::Ui) {
        let mut outbox = Vec::new();
        view(&self.model, ui, &mut outbox);
        for message in outbox {
            self.receive(message);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn hide() -> WindowEffect {
        WindowEffect::Hide {
            title: labels::APP_TITLE.into(),
        }
    }

    fn show() -> WindowEffect {
        WindowEffect::Show {
            title: labels::TRAY_MENU_QUIT.into(),
        }
    }

    fn show_config() -> WindowEffect {
        WindowEffect::Show {
            title: labels::CONFIG_WINDOW_TITLE.into(),
        }
    }

    #[test]
    fn eframe_forced_show_is_undone_at_second_tick() {
        let mut runtime = Runtime::new(Model::default());
        // Premier cycle : eframe n'a pas encore forcé l'affichage.
        let effects = runtime.step(false);
        assert!(!effects.contains(&hide()), "rien à cacher au premier tick");
        // Deuxième cycle : la fenêtre racine disparaît pour de bon.
        let effects = runtime.step(false);
        assert!(
            effects.contains(&hide()),
            "la racine doit être cachée au tick 2"
        );
    }

    #[test]
    fn quit_request_shows_the_window() {
        let mut runtime = Runtime::new(Model::default());
        runtime.receive(Message::QuitRequested);
        let effects = runtime.step(false);
        assert!(
            effects.contains(&show()),
            "la demande de confirmation doit montrer la fenêtre"
        );
        assert!(
            effects.contains(&WindowEffect::Repaint),
            "une fenêtre cachée ne se réveille pas sans repaint"
        );
        assert!(runtime.model().quit_request.is_some());
    }

    #[test]
    fn user_close_is_cancelled_and_never_quits() {
        let mut runtime = Runtime::new(Model::default());
        runtime.receive(Message::QuitRequested);
        runtime.step(false);
        // L'utilisateur clique ✕ pendant la confirmation.
        let effects = runtime.step(true);
        assert!(
            effects.contains(&WindowEffect::CancelClose),
            "le ✕ de l'utilisateur doit être annulé (✕ ≡ Non)"
        );
        assert!(
            !effects.contains(&WindowEffect::Close),
            "le ✕ ne quitte jamais"
        );
        assert!(
            runtime.model().quit_request.is_none(),
            "« Non » referme la demande"
        );
    }

    #[test]
    fn config_request_shows_the_window_with_config_title() {
        let mut runtime = Runtime::new(Model::default());
        runtime.step(false); // anti-flash : la racine se cache au tick 2
        runtime.step(false);
        runtime.receive(Message::ConfigOpenRequested);
        let effects = runtime.step(false);
        assert!(
            effects.contains(&show_config()),
            "ouvrir la configuration doit montrer la fenêtre, titrée pour elle (UC-04, étape 2)"
        );
        assert!(effects.contains(&WindowEffect::Repaint));
        assert!(runtime.model().config_window.is_some());
    }

    #[test]
    fn save_request_becomes_a_save_effect_and_close_waits_for_its_result() {
        let mut runtime = Runtime::new(Model::default());
        runtime.step(false); // anti-flash des ticks 1-2 consommé :
        runtime.step(false); //  le test ouvre ensuite, comme en vrai
        runtime.receive(Message::ConfigOpenRequested);
        runtime.step(false); // la fenêtre se montre
        runtime.receive(Message::TestParamChanged(true));
        runtime.receive(Message::ConfigSaveRequested);
        // Étapes 4 → 5 : la commande de persistance part au harnais,
        // la fenêtre attend le résultat — pas de refermeture anticipée.
        let effects = runtime.step(false);
        assert_eq!(
            effects,
            vec![WindowEffect::SaveConfig {
                saved: SavedConfig { test_param: true }
            }],
            "« Enregistrer » doit ordonner la persistance avec les valeurs au clic"
        );
        assert!(
            !effects
                .iter()
                .any(|effect| matches!(effect, WindowEffect::Hide { .. })),
            "la fenêtre ne se referme qu'au résultat de la persistance (étapes 5 → 6)"
        );
        // Le harnais rapporte le succès : la fenêtre se referme alors.
        runtime.receive(Message::ConfigSaved);
        let effects = runtime.step(false);
        assert!(
            effects
                .iter()
                .any(|effect| matches!(effect, WindowEffect::Hide { .. })),
            "persistance réussie : la fenêtre se referme (étape 6)"
        );
        assert!(
            !effects.contains(&WindowEffect::Close),
            "la configuration n'arrête jamais l'application"
        );
    }

    #[test]
    fn user_close_cancels_config_without_exit() {
        let mut runtime = Runtime::new(Model::default());
        runtime.receive(Message::ConfigOpenRequested);
        runtime.step(false);
        runtime.receive(Message::TestParamChanged(true));
        // L'utilisateur clique ✕ pendant la configuration.
        let effects = runtime.step(true);
        assert!(
            effects.contains(&WindowEffect::CancelClose),
            "le ✕ de l'utilisateur doit être annulé (garde du harnais)"
        );
        assert!(
            !effects.contains(&WindowEffect::Close),
            "le ✕ ne quitte jamais"
        );
        assert!(
            runtime.model().config_window.is_none(),
            "le ✕ referme la configuration (✕ ≡ Annuler, extension 4a)"
        );
        assert!(
            !runtime.model().test_param,
            "le ✕ referme sans enregistrer : retour à la valeur d'ouverture"
        );
    }

    #[test]
    fn confirmed_quit_orders_close_and_survives_reflection() {
        let mut runtime = Runtime::new(Model::default());
        runtime.receive(Message::QuitRequested);
        runtime.receive(Message::QuitConfirmed);
        // L'arrêt ordonné devient un effet Close.
        let effects = runtime.step(false);
        assert!(
            effects.contains(&WindowEffect::Close),
            "« Oui » doit ordonner la fermeture"
        );
        assert!(!effects.contains(&WindowEffect::CancelClose));
        // winit renvoie ce Close sous forme d'évènement à la frame
        // suivante : il ne doit PAS être annulé (bug de recette UC-05).
        let effects = runtime.step(true);
        assert!(
            !effects.contains(&WindowEffect::CancelClose),
            "la fermeture ordonnée par nous-même ne doit jamais être annulée"
        );
    }

    #[test]
    fn cancelled_request_hides_the_window_without_close() {
        let mut runtime = Runtime::new(Model::default());
        runtime.receive(Message::QuitRequested);
        runtime.step(false);
        runtime.receive(Message::QuitCancelled);
        let effects = runtime.step(false);
        assert!(effects.contains(&hide()), "« Non » referme la fenêtre");
        assert!(
            !effects.contains(&WindowEffect::Close),
            "« Non » n'ordonne jamais la fermeture"
        );
    }

    /// Contrat de construction : `Runtime::default()` et
    /// `Runtime::new(Model::default())` doivent être comportementalement
    /// indiscernables. Tautologique tant que `Default` délègue à
    /// `new()` — c'est un filet de sécurité : le jour où les deux
    /// chemins deviennent indépendants (ex. `derive(Default)` sans
    /// inversion de délégation, ou impl manuelle qui dérive), la
    /// moindre divergence d'état initial doit tomber rouge ici
    /// (vérifié par mutation sur `logic_ticks`).
    #[test]
    fn default_construction_behaves_like_new() {
        fn scenario(runtime: &mut Runtime) -> Vec<Vec<WindowEffect>> {
            let mut trace = vec![runtime.step(false)];
            runtime.receive(Message::QuitRequested);
            trace.push(runtime.step(false));
            trace.push(runtime.step(true)); // ✕ ≡ « Non »
            runtime.receive(Message::QuitConfirmed);
            trace.push(runtime.step(false)); // arrêt ordonné
            trace.push(runtime.step(true)); // fermeture réfléchie, pas annulée
            trace
        }

        let mut via_new = Runtime::new(Model::default());
        let mut via_default = Runtime::default();
        assert_eq!(
            scenario(&mut via_new),
            scenario(&mut via_default),
            "Runtime::default() et Runtime::new(Model::default()) doivent être indiscernables \
             (même état initial, même comportement)"
        );
    }
}
