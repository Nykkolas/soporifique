//! Soporifique : surveille le niveau de batterie et met le portable en
//! hibernation au franchissement d'un seuil configurable.
//!
//! Ce fichier est le **harnais** (pattern TEA imposé par la structure,
//! cf. `docs/RUST-TEA.md`) : câblage eframe (fenêtre) + ksni (icône de
//! barre d'état) et exécution des [`Command`]. Toute la logique vit dans
//! la lib — le harnais n'est pas couvert en test (plafond structurel
//! déclaré dans `docs/COVERAGE.md`).
//!
//! Modèle de fenêtres : l'application vit dans la barre d'état ; sa
//! fenêtre n'est montrée QUE pour porter une demande à l'utilisateur
//! (aujourd'hui la confirmation d'arrêt UC-05 — un jour la fenêtre de
//! configuration UC-04). Le multi-viewport a été essayé et abandonné :
//! la création de fenêtre enfant pendant une passe cachée échoue en
//! silence sous X11/GL puis panique (cf. historique de session).

mod tray;

use std::sync::mpsc::{self, Receiver};

use eframe::egui;
use soporifique::command::Command;
use soporifique::labels;
use soporifique::message::Message;
use soporifique::model::Model;
use soporifique::update::update;
use soporifique::view::view;

fn main() -> eframe::Result {
    // Fenêtre unique, cachée au départ : elle n'existe à l'écran que
    // pendant une demande (confirmation UC-04/05).
    let viewport = egui::ViewportBuilder::default()
        .with_visible(false)
        .with_inner_size([420.0, 170.0])
        .with_resizable(false)
        .with_title(labels::APP_TITLE);
    let options = eframe::NativeOptions {
        viewport,
        ..Default::default()
    };

    eframe::run_native(
        labels::APP_TITLE,
        options,
        Box::new(|cc| Ok(Box::new(Soporifique::new(cc.egui_ctx.clone())?))),
    )
}

/// L'application eframe : détient le modèle, collecte les Messages de
/// tous les producteurs (tray, vue), exécute les [`Command`] de
/// [`update`] — et rien d'autre.
struct Soporifique {
    model: Model,
    postbox: Receiver<Message>,
    tray: ksni::blocking::Handle<tray::SoporifiqueTray>,
    pending_commands: Vec<Command>,
    // Compteur de ticks d'`App::logic` — cf. `hide_root_forever`.
    logic_ticks: u32,
    // Le premier re-cachage d'eframe est-il fait ?
    root_hidden_once: bool,
    // La fenêtre porte-t-elle une demande à l'écran ?
    window_shown: bool,
    // Fermeture ordonnée par l'application elle-même (UC-05, étape 6) :
    // winit renvoie notre ViewportCommand::Close sous forme d'évènement
    // Close en entrée — ce drapeau le distingue du ✕ de l'utilisateur
    // (sans lui, la garde « ✕ ≡ Non » annule notre propre arrêt : bug
    // de recette UC-05, cf. docs/TODO.md et la spec test).
    self_close: bool,
}

impl Soporifique {
    fn new(ctx: egui::Context) -> Result<Self, ksni::Error> {
        let (sender, receiver) = mpsc::channel();
        // L'icône et son menu démarrent avec l'application (précondition
        // UC-05 : « lancée en mode barre d'état système »).
        use ksni::blocking::TrayMethods as _;
        let tray = tray::SoporifiqueTray::new(sender, ctx).spawn()?;
        Ok(Self {
            model: Model::default(),
            postbox: receiver,
            tray,
            pending_commands: Vec::new(),
            logic_ticks: 0,
            root_hidden_once: false,
            window_shown: false,
            self_close: false,
        })
    }

    /// Un producteur a émis : `update` pur, les Command vont dans la
    /// file d'attente du harnais (jamais exécutées dans `update`).
    fn receive(&mut self, message: Message) {
        let (model, commands) = update(self.model.clone(), message);
        self.model = model;
        self.pending_commands.extend(commands);
    }

    /// eframe force l'affichage de la fenêtre après le premier rendu
    /// (anti-flash blanc, `eframe::native::epi_integration::post_rendering`)
    /// : la recacher une fois ce premier rendu passé (tick ≥ 2). Le reste
    /// de la vie de la fenêtre est géré par `sync_window_visibility`.
    fn hide_root_forever(&mut self, ctx: &egui::Context) {
        self.logic_ticks = self.logic_ticks.saturating_add(1);
        if self.logic_ticks >= 2 && !self.root_hidden_once {
            self.root_hidden_once = true;
            ctx.send_viewport_cmd(egui::ViewportCommand::Visible(false));
        }
    }

    /// Synchronise la fenêtre unique avec le modèle :
    /// montrée (et titrée) pour porter la demande de confirmation,
    /// cachée dès qu'il n'y a plus rien à demander.
    /// `request_repaint` porte la passe qui suit chaque transition —
    /// eframe ne réveille pas tout seul une fenêtre cachée.
    fn sync_window_visibility(&mut self, ctx: &egui::Context) {
        let confirmation_pending = self.model.quit_request.is_some();
        if confirmation_pending && !self.window_shown {
            self.window_shown = true;
            ctx.send_viewport_cmd(egui::ViewportCommand::Title(
                labels::TRAY_MENU_QUIT.into(),
            ));
            ctx.send_viewport_cmd(egui::ViewportCommand::Visible(true));
            ctx.request_repaint();
        } else if !confirmation_pending && self.window_shown {
            self.window_shown = false;
            ctx.send_viewport_cmd(egui::ViewportCommand::Visible(false));
            ctx.send_viewport_cmd(egui::ViewportCommand::Title(
                labels::APP_TITLE.into(),
            ));
            ctx.request_repaint();
        }
    }
}

impl eframe::App for Soporifique {
    fn logic(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        // 1. Producteur tray : vider la boîte à lettres du cycle.
        // (App::logic tourne même fenêtre cachée : l'icône peut appeler
        // l'application à tout instant, cf. docs/RUST-EGUI.md repaint.)
        while let Ok(message) = self.postbox.try_recv() {
            self.receive(message);
        }

        // 2. Évènement de fenêtre Close : n'est pas « Non » si c'est nous
        // qui avons ordonné la fermeture (le drapeau départage les deux,
        // winit les rendant indistinguables). Fermer ne quitte jamais
        // l'application de la main de l'utilisateur (règle UC-04,
        // extension 5a de UC-05).
        let close_requested = ctx.input(|input| {
            input
                .viewport()
                .events
                .iter()
                .any(|event| matches!(event, egui::ViewportEvent::Close))
        });
        if close_requested && !self.self_close {
            ctx.send_viewport_cmd(egui::ViewportCommand::CancelClose);
            self.receive(Message::QuitCancelled);
        }

        // 3. Infrastructure : la fenêtre vit sa vie (cachée, ou porteuse
        // de la demande), cf. méthodes.
        self.hide_root_forever(ctx);
        self.sync_window_visibility(ctx);

        // 4. Exécution des Command ordonnées par `update` — jamais
        // dans `update` lui-même (cf. docs/RUST-TEA.md).
        for command in std::mem::take(&mut self.pending_commands) {
            match command {
                Command::ExitApplication => {
                    // UC-05, étape 6 : fermer la fenêtre racine arrête
                    // l'application eframe (et l'icône tray via le Drop).
                    // Marquer la fermeture comme la nôtre AVANT qu'elle
                    // revienne en évènement (cf. garde ci-dessus).
                    self.self_close = true;
                    ctx.send_viewport_cmd(egui::ViewportCommand::Close);
                }
            }
        }
    }

    fn ui(&mut self, ui: &mut egui::Ui, _frame: &mut eframe::Frame) {
        // Vue : rend le modèle, ramasse ses Messages (la fenêtre, quand
        // elle est montrée, ne porte que ce que le modèle lui donne).
        let mut outbox = Vec::new();
        view(&self.model, ui, &mut outbox);
        for message in outbox {
            self.receive(message);
        }
    }
}

impl Drop for Soporifique {
    fn drop(&mut self) {
        // Retrait propre de l'icône de la barre d'état (étape 6 :
        // « disparaît de la barre d'état système »).
        self.tray.shutdown().wait();
    }
}
