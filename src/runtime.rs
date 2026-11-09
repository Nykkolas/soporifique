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
    command::Command, labels, message::Message, model::Model, update::update, view::view,
};

/// Effet de fenêtre ordonné au harnais — jamais exécuté ici.
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
    /// Annuler une fermeture demandée (garde « ✕ ≡ Non » : fermer ne
    /// quitte jamais l'application de la main de l'utilisateur — règle
    /// UC-04, extension 5a de UC-05).
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
        Self::new()
    }
}

impl Runtime {
    pub fn new() -> Self {
        Self {
            model: Model::default(),
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

        // 1. Garde « ✕ ≡ Non » : la fermeture demandée par l'utilisateur
        //    est annulée — sauf celle que nous avons ordonnée.
        if close_requested && !self.self_close {
            effects.push(WindowEffect::CancelClose);
            self.receive(Message::QuitCancelled);
        }

        // 2. eframe force l'affichage de la fenêtre après le premier
        //    rendu (anti-flash blanc,
        //    `eframe::native::epi_integration::post_rendering`) : la
        //    recacher une fois ce rendu passé (tick ≥ 2).
        self.logic_ticks = self.logic_ticks.saturating_add(1);
        if self.logic_ticks >= 2 && !self.root_hidden_once {
            self.root_hidden_once = true;
            effects.push(WindowEffect::Hide {
                title: labels::APP_TITLE.into(),
            });
        }

        // 3. La fenêtre ne vit à l'écran que pour porter une demande
        //    (aujourd'hui la confirmation d'arrêt, un jour la
        //    configuration UC-04). `Repaint` porte la passe qui suit :
        //    une fenêtre cachée ne se réveille pas toute seule.
        let confirmation_pending = self.model.quit_request.is_some();
        if confirmation_pending && !self.window_shown {
            self.window_shown = true;
            effects.push(WindowEffect::Show {
                title: labels::TRAY_MENU_QUIT.into(),
            });
            effects.push(WindowEffect::Repaint);
        } else if !confirmation_pending && self.window_shown {
            self.window_shown = false;
            effects.push(WindowEffect::Hide {
                title: labels::APP_TITLE.into(),
            });
        }

        // 4. Les Command ordonnées par `update` deviennent des effets —
        //    jamais exécutées dans `update`, ni ici.
        for command in std::mem::take(&mut self.pending_commands) {
            match command {
                Command::ExitApplication => {
                    self.self_close = true;
                    effects.push(WindowEffect::Close);
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

    #[test]
    fn eframe_forced_show_is_undone_at_second_tick() {
        let mut runtime = Runtime::new();
        // Premier cycle : eframe n'a pas encore forcé l'affichage.
        let effects = runtime.step(false);
        assert!(!effects.contains(&hide()), "rien à cacher au premier tick");
        // Deuxième cycle : la fenêtre racine disparaît pour de bon.
        let effects = runtime.step(false);
        assert!(effects.contains(&hide()), "la racine doit être cachée au tick 2");
    }

    #[test]
    fn quit_request_shows_the_window() {
        let mut runtime = Runtime::new();
        runtime.receive(Message::QuitRequested);
        let effects = runtime.step(false);
        assert!(effects.contains(&show()), "la demande de confirmation doit montrer la fenêtre");
        assert!(
            effects.contains(&WindowEffect::Repaint),
            "une fenêtre cachée ne se réveille pas sans repaint"
        );
        assert!(runtime.model().quit_request.is_some());
    }

    #[test]
    fn user_close_is_cancelled_and_never_quits() {
        let mut runtime = Runtime::new();
        runtime.receive(Message::QuitRequested);
        runtime.step(false);
        // L'utilisateur clique ✕ pendant la confirmation.
        let effects = runtime.step(true);
        assert!(
            effects.contains(&WindowEffect::CancelClose),
            "le ✕ de l'utilisateur doit être annulé (✕ ≡ Non)"
        );
        assert!(!effects.contains(&WindowEffect::Close), "le ✕ ne quitte jamais");
        assert!(runtime.model().quit_request.is_none(), "« Non » referme la demande");
    }

    #[test]
    fn confirmed_quit_orders_close_and_survives_reflection() {
        let mut runtime = Runtime::new();
        runtime.receive(Message::QuitRequested);
        runtime.receive(Message::QuitConfirmed);
        // L'arrêt ordonné devient un effet Close.
        let effects = runtime.step(false);
        assert!(effects.contains(&WindowEffect::Close), "« Oui » doit ordonner la fermeture");
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
        let mut runtime = Runtime::new();
        runtime.receive(Message::QuitRequested);
        runtime.step(false);
        runtime.receive(Message::QuitCancelled);
        let effects = runtime.step(false);
        assert!(effects.contains(&hide()), "« Non » referme la fenêtre");
        assert!(!effects.contains(&WindowEffect::Close), "« Non » n'ordonne jamais la fermeture");
    }
}
