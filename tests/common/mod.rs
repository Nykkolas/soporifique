//! Hôte OS simulé autour du vrai [`Runtime`], partagé par les tests de
//! spec E2E (`tests/uc0*.rs`) : une frame = logique (décisions du
//! runtime, code de production) puis vue.
//!
//! Le revers du mode d'organisation de `tests/` : chaque fichier y
//! compile en crate séparé, d'où ce module partagé par `mod common;`
//! (convention Rust : un répertoire n'est pas un binaire de test).
//!
//! Seule la partie OS vit ici (miroir winit) : fermer une vraie
//! fenêtre Cinnamon n'est pas rejouable headless — le test vérifie les
//! ordres (`WindowEffect`) confiés au harnais, pas leur exécution
//! réelle. Exception : la persistance des paramètres (UC-04, étape 5)
//! s'exécute **réellement** sur le fichier fourni — le disque est la
//! vérité extérieure que la spec doit lire. La vérité du tray (DBus)
//! reste la recette manuelle.

use std::path::Path;

use egui_kittest::Harness;
use soporifique::message::Message;
use soporifique::model::Model;
use soporifique::runtime::{Runtime, WindowEffect};
use soporifique::store::{ConfigStore as _, FileConfigStore, boot_model};

/// L'hôte simulé : miroir du comportement de winit autour du runtime.
pub struct Host {
    /// Le vrai runtime de l'application, booté comme au lancement.
    pub runtime: Runtime,
    /// Miroir winit : évènement `Close` en attente pour la frame
    /// suivante (✕ utilisateur OU notre propre fermeture réfléchie).
    pub reflected_close: bool,
    /// Journal des effets ordonnés par le runtime.
    pub ordered_effects: Vec<WindowEffect>,
    /// Miroir winit : sans CancelClose, une fermeture demandée termine
    /// l'application.
    pub exited: bool,
    /// Persistance des paramètres (UC-04, étape 5) : réelle si un
    /// fichier est fourni, `None` pour un lancement sans disque.
    store: Option<FileConfigStore>,
    /// Résultats de persistance en attente : exécutés en fin de frame,
    /// ils entrent au début de la suivante (même asynchronisme que la
    /// boîte à lettres du tray dans main.rs).
    pending_feedback: Vec<Message>,
}

// Chaque binaire de test n'utilise qu'une partie des constructeurs
// (UC-05 : `new` ; UC-04 : `booted`/`booted_harness`) — les limbes
// sont normales, le module est partagé par des crates séparés.
#[allow(dead_code)]
impl Host {
    /// Lancement sans configuration persistée (UC-05 : aucune
    /// sauvegarde en jeu — un ordre de persistance sur ce host
    /// échouerait explicitement).
    pub fn new() -> Self {
        Self::bare(Runtime::new(Model::default()))
    }

    fn bare(runtime: Runtime) -> Self {
        Self {
            runtime,
            reflected_close: false,
            ordered_effects: Vec::new(),
            exited: false,
            store: None,
            pending_feedback: Vec::new(),
        }
    }

    /// Lancement de l'application avec configuration persistée :
    /// relit le fichier au boot (cf. `store::boot_model`) et exécute
    /// réellement les sauvegardes dessus.
    pub fn booted(path: impl AsRef<Path>) -> Self {
        let store = FileConfigStore::new(path.as_ref().to_path_buf());
        let (model, warning) = boot_model(&store);
        assert!(
            warning.is_none(),
            "les specs bootent sur un fichier sain : {warning:?}"
        );
        let mut host = Self::bare(Runtime::new(model));
        host.store = Some(store);
        host
    }

    /// Harnais kittest booté sur le disque (raccourci des specs :
    /// lancer l'application comme `cargo run` le ferait).
    pub fn booted_harness(path: impl AsRef<Path>) -> Harness<'static, Self> {
        Harness::new_ui_state(
            |ui, host: &mut Self| host.frame(ui),
            Self::booted(path),
        )
    }

    /// Une frame : logique du runtime puis vue (même ordre que main.rs).
    pub fn frame(&mut self, ui: &mut egui::Ui) {
        // 1. Les résultats de persistance de la frame précédente
        //    entrent premiers (boîte à lettres du harnais).
        for message in std::mem::take(&mut self.pending_feedback) {
            self.runtime.receive(message);
        }

        let close_requested = self.reflected_close;
        self.reflected_close = false;

        // 2. Décisions du runtime, exécution des effets OS.
        let effects = self.runtime.step(close_requested);
        self.ordered_effects.extend(effects.iter().cloned());
        if close_requested && !effects.contains(&WindowEffect::CancelClose) {
            self.exited = true;
        }
        for effect in effects {
            match effect {
                WindowEffect::SaveConfig { saved } => {
                    // Vérité OS : le vrai disque — même rapport que
                    // main.rs (chaque Command rapporte).
                    let report = match &mut self.store {
                        Some(store) => match store.save_saved(&saved) {
                            Ok(()) => Message::ConfigSaved,
                            Err(error) => Message::ConfigSaveFailed(error),
                        },
                        None => {
                            Message::ConfigSaveFailed("no config store on this host".into())
                        }
                    };
                    self.pending_feedback.push(report);
                }
                WindowEffect::Close => self.reflected_close = true,
                _ => {}
            }
        }

        // 3. Vue : rend le modèle, émet ses Messages.
        self.runtime.render(ui);
    }
}
