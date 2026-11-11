//! Soporifique : surveille le niveau de batterie et met le portable en
//! hibernation au franchissement d'un seuil configurable.
//!
//! Ce fichier est le **harnais** (pattern TEA imposé par la structure,
//! cf. `docs/RUST-TEA.md`) : câblage eframe (fenêtre) + ksni (icône de
//! barre d'état), et traduction des [`WindowEffect`] ordonnés par le
//! runtime en commandes egui. Les décisions vivent dans
//! `soporifique::runtime` (lib, testé) ; ce binaire n'est que de la
//! colle — c'est la partie non couvrable en test (plafond structurel
//! déclaré dans `docs/COVERAGE.md`).
//!
//! Modèle de fenêtres : l'application vit dans la barre d'état ; sa
//! fenêtre n'est montrée QUE pour porter une demande à l'utilisateur
//! (confirmation d'arrêt UC-05, fenêtre de configuration UC-04).

mod tray;

use std::path::PathBuf;
use std::sync::mpsc::{self, Receiver};

use eframe::egui;
use soporifique::message::Message;
use soporifique::runtime::{Runtime, WindowEffect};
use soporifique::store::{ConfigStore as _, FileConfigStore, boot_model};

/// Fichier de configuration : `config` sous `$XDG_CONFIG_HOME/soporifique`
/// (défaut `$HOME/.config/soporifique/config`) — colle du harnais, le
/// cœur ne connaît ni chemin ni disque (cf. `store.rs`).
fn config_path() -> PathBuf {
    let config_home = std::env::var_os("XDG_CONFIG_HOME")
        .map(PathBuf::from)
        .or_else(|| {
            std::env::var_os("HOME").map(|home| PathBuf::from(home).join(".config"))
        })
        .unwrap_or_else(|| PathBuf::from(".config"));
    config_home.join(env!("CARGO_PKG_NAME")).join("config")
}

fn main() -> eframe::Result {
    // Fenêtre unique, cachée au départ : elle n'existe à l'écran que
    // pendant une demande (cf. `runtime::Runtime::step`).
    let viewport = egui::ViewportBuilder::default()
        .with_visible(false)
        .with_inner_size([420.0, 170.0])
        .with_resizable(false)
        .with_title(soporifique::labels::APP_TITLE);
    let options = eframe::NativeOptions {
        viewport,
        ..Default::default()
    };

    eframe::run_native(
        soporifique::labels::APP_TITLE,
        options,
        Box::new(|cc| Ok(Box::new(Soporifique::new(cc.egui_ctx.clone())?))),
    )
}

/// L'application eframe : câble le runtime aux producteurs de Messages
/// (tray, vue), traduit ses effets et exécute le stockage — rien d'autre.
struct Soporifique {
    runtime: Runtime,
    postbox: Receiver<Message>,
    tray: ksni::blocking::Handle<tray::SoporifiqueTray>,
    /// Persistance des paramètres (UC-04, étape 5) : injectée, le
    /// cœur ne lit jamais le disque lui-même.
    store: FileConfigStore,
}

impl Soporifique {
    fn new(ctx: egui::Context) -> Result<Self, ksni::Error> {
        let (sender, receiver) = mpsc::channel();
        // L'icône et son menu démarrent avec l'application (précondition
        // UC-05 : « lancée en mode barre d'état système »).
        use ksni::blocking::TrayMethods as _;
        let tray = tray::SoporifiqueTray::new(sender, ctx).spawn()?;

        // Lancement : les paramètres persistés au lancement précédent
        // sont relus ici (UC-04). Échec de lecture : l'application
        // démarre quand même sur les valeurs par défaut — le journal
        // détaillé vivra avec son UC (garantie minimale UC-04).
        let store = FileConfigStore::new(config_path());
        let (model, warning) = boot_model(&store);
        if let Some(warning) = warning {
            eprintln!("{}: {warning}", soporifique::labels::APP_TITLE);
        }

        Ok(Self {
            runtime: Runtime::new(model),
            postbox: receiver,
            tray,
            store,
        })
    }
}

impl Soporifique {
    /// Traduction d'un effet du runtime — l'unique connaissance de
    /// winit/eframe/disque de ce binaire.
    fn apply(&mut self, effect: WindowEffect, ctx: &egui::Context) {
        match effect {
            WindowEffect::Show { title } => {
                ctx.send_viewport_cmd(egui::ViewportCommand::Title(title));
                ctx.send_viewport_cmd(egui::ViewportCommand::Visible(true));
            }
            WindowEffect::Hide { title } => {
                ctx.send_viewport_cmd(egui::ViewportCommand::Visible(false));
                ctx.send_viewport_cmd(egui::ViewportCommand::Title(title));
            }
            // UC-04, étape 5 : exécution réelle de la persistance sur
            // le stockage injecté, puis rapport du résultat au cœur
            // (chaque Command rapporte — la fenêtre se referme au
            // succès seulement, l'échec s'y affiche).
            WindowEffect::SaveConfig { saved } => {
                let report = match self.store.save_saved(&saved) {
                    Ok(()) => Message::ConfigSaved,
                    Err(error) => Message::ConfigSaveFailed(error),
                };
                self.runtime.receive(report);
            }
            WindowEffect::CancelClose => {
                ctx.send_viewport_cmd(egui::ViewportCommand::CancelClose)
            }
            WindowEffect::Close => ctx.send_viewport_cmd(egui::ViewportCommand::Close),
            WindowEffect::Repaint => ctx.request_repaint(),
        }
    }
}

impl eframe::App for Soporifique {
    fn logic(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        // 1. Producteur tray : vider la boîte à lettres du cycle
        //    (App::logic tourne même fenêtre cachée).
        while let Ok(message) = self.postbox.try_recv() {
            self.runtime.receive(message);
        }

        // 2. Décisions du runtime (pures, testées dans la lib), effets
        //    traduits en commandes egui — et persistance exécutée.
        let close_requested = ctx.input(|input| {
            input
                .viewport()
                .events
                .iter()
                .any(|event| matches!(event, egui::ViewportEvent::Close))
        });
        for effect in self.runtime.step(close_requested) {
            self.apply(effect, ctx);
        }
    }

    fn ui(&mut self, ui: &mut egui::Ui, _frame: &mut eframe::Frame) {
        // Vue : rend le modèle, émet ses Messages (via le runtime).
        self.runtime.render(ui);
    }
}

impl Drop for Soporifique {
    fn drop(&mut self) {
        // Retrait propre de l'icône de la barre d'état (UC-05, étape 6 :
        // « disparaît de la barre d'état système »).
        self.tray.shutdown().wait();
    }
}
