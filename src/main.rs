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
//! (aujourd'hui la confirmation d'arrêt UC-05 — un jour la fenêtre de
//! configuration UC-04).

mod tray;

use std::sync::mpsc::{self, Receiver};

use eframe::egui;
use soporifique::message::Message;
use soporifique::runtime::{Runtime, WindowEffect};

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
/// (tray, vue) et traduit ses effets — et rien d'autre.
struct Soporifique {
    runtime: Runtime,
    postbox: Receiver<Message>,
    tray: ksni::blocking::Handle<tray::SoporifiqueTray>,
}

impl Soporifique {
    fn new(ctx: egui::Context) -> Result<Self, ksni::Error> {
        let (sender, receiver) = mpsc::channel();
        // L'icône et son menu démarrent avec l'application (précondition
        // UC-05 : « lancée en mode barre d'état système »).
        use ksni::blocking::TrayMethods as _;
        let tray = tray::SoporifiqueTray::new(sender, ctx).spawn()?;
        Ok(Self {
            runtime: Runtime::new(),
            postbox: receiver,
            tray,
        })
    }
}

/// Traduction d'un effet du runtime en commandes egui — l'unique
/// connaissance de winit/eframe de ce binaire.
fn apply(effect: WindowEffect, ctx: &egui::Context) {
    match effect {
        WindowEffect::Show { title } => {
            ctx.send_viewport_cmd(egui::ViewportCommand::Title(title));
            ctx.send_viewport_cmd(egui::ViewportCommand::Visible(true));
        }
        WindowEffect::Hide { title } => {
            ctx.send_viewport_cmd(egui::ViewportCommand::Visible(false));
            ctx.send_viewport_cmd(egui::ViewportCommand::Title(title));
        }
        WindowEffect::CancelClose => {
            ctx.send_viewport_cmd(egui::ViewportCommand::CancelClose)
        }
        WindowEffect::Close => ctx.send_viewport_cmd(egui::ViewportCommand::Close),
        WindowEffect::Repaint => ctx.request_repaint(),
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
        //    traduits en commandes egui.
        let close_requested = ctx.input(|input| {
            input
                .viewport()
                .events
                .iter()
                .any(|event| matches!(event, egui::ViewportEvent::Close))
        });
        for effect in self.runtime.step(close_requested) {
            apply(effect, ctx);
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
