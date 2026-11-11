//! Icône de la barre d'état système (ksni/DBus) — détail d'infrastructure
//! du harnais, non couvert en test (cf. `docs/RUST-EGUI.md` et la
//! cartouche de `tests/uc05_quitter_application.rs`).
//!
//! Le tray est un producteur de Messages comme un autre : il émet, la
//! garde vit dans `update` — aucune logique ici.

use ksni::{Icon, Tray, menu::StandardItem};
use std::sync::mpsc::Sender;

use soporifique::labels;
use soporifique::message::Message;

/// L'icône : un croissant de lune — Soporifique veille pendant que la
/// machine s'endort.
pub struct SoporifiqueTray {
    /// Boîte à lettres vers le harnais egui : le tray n'envoie que des
    /// Messages (UC-05, étape 3).
    postbox: Sender<Message>,
    /// Pour réveiller l'UI endormie au moment d'émettre (egui ne redessine
    /// que sur interaction, cf. `docs/RUST-EGUI.md`).
    ctx: egui::Context,
}

impl SoporifiqueTray {
    pub fn new(postbox: Sender<Message>, ctx: egui::Context) -> Self {
        Self { postbox, ctx }
    }

    /// Émet un Message vers le harnais et réveille l'UI.
    fn post(&self, message: Message) {
        // Un harnais parti ne consomme plus sa boîte à lettres : ignorable.
        let _ = self.postbox.send(message);
        self.ctx.request_repaint();
    }
}

impl Tray for SoporifiqueTray {
    fn id(&self) -> String {
        env!("CARGO_PKG_NAME").into()
    }

    fn title(&self) -> String {
        // Infobulle de l'icône : terme du glossaire (libellé app).
        labels::APP_TITLE.into()
    }

    fn icon_pixmap(&self) -> Vec<Icon> {
        vec![moon_icon()]
    }

    /// Clic gauche sur l'icône (signal `Activate`) : UC-04, étape 1 —
    /// émettre, c'est tout.
    fn activate(&mut self, _x: i32, _y: i32) {
        self.post(Message::ConfigOpenRequested);
    }

    fn menu(&self) -> Vec<ksni::MenuItem<Self>> {
        // Les décisions (libellés, Messages, noms d'icônes) vivent dans
        // le modèle de menu de la lib (`tray_menu`) : cette colle
        // n'expose que le transport DBus — aucune décision ici.
        soporifique::tray_menu::menu()
            .into_iter()
            .map(|entry| {
                let message = entry.message;
                StandardItem {
                    label: entry.label.into(),
                    icon_name: entry.icon.into(),
                    activate: Box::new(move |tray: &mut Self| {
                        // Émettre, c'est tout (UC-04 1a.3 / UC-05 étape 3).
                        tray.post(message.clone());
                    }),
                    ..Default::default()
                }
                .into()
            })
            .collect()
    }
}

/// Croissant de lune indigo sur fond transparent, 64×64, ARGB32 en
/// réseau (format StatusNotifierItem). Généré au lancement — aucun
/// asset à embarquer, aucune dépendance image.
fn moon_icon() -> Icon {
    const SIZE: i32 = 64;
    const MOON_RADIUS: f32 = 22.0;
    const BITE_RADIUS: f32 = 17.0;
    const INDIGO: [u8; 4] = [255, 77, 92, 161]; // ARGB
    const TRANSPARENT: [u8; 4] = [0, 0, 0, 0];

    let mut data = Vec::with_capacity((SIZE * SIZE * 4) as usize);
    for y in 0..SIZE {
        for x in 0..SIZE {
            let x = x as f32;
            let y = y as f32;
            let dx = x - 27.0;
            let dy = y - 32.0;
            let in_moon = dx * dx + dy * dy <= MOON_RADIUS * MOON_RADIUS;
            let bx = x - 39.0;
            let by = y - 23.0;
            let in_bite = bx * bx + by * by < BITE_RADIUS * BITE_RADIUS;
            let pixel = if in_moon && !in_bite {
                INDIGO
            } else {
                TRANSPARENT
            };
            data.extend_from_slice(&pixel);
        }
    }
    Icon {
        width: SIZE,
        height: SIZE,
        data,
    }
}
