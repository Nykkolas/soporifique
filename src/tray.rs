//! Icône de la barre d'état système (ksni/DBus) — détail d'infrastructure
//! du harnais, non couvert en test (cf. `docs/RUST-EGUI.md` et la
//! cartouche de `tests/uc05_quitter_application.rs`).
//!
//! Le tray est un producteur de Messages comme un autre : il émet, la
//! garde vit dans `update` — aucune logique ici.

use ksni::{menu::StandardItem, Icon, Tray};
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

    fn menu(&self) -> Vec<ksni::MenuItem<Self>> {
        vec![
            StandardItem {
                label: labels::TRAY_MENU_QUIT.into(),
                // Icône standard freedesktop « quitter » (UC-05 étape 2 :
                // « un icône parlant » ; convention plateforme d'abord,
                // cf. docs/UX-GENERIC.md).
                icon_name: "application-exit".into(),
                activate: Box::new(|tray: &mut Self| {
                    // UC-05, étape 3 : émettre, c'est tout.
                    tray.post(Message::QuitRequested);
                }),
                ..Default::default()
            }
            .into(),
        ]
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
            let pixel = if in_moon && !in_bite { INDIGO } else { TRANSPARENT };
            data.extend_from_slice(&pixel);
        }
    }
    Icon {
        width: SIZE,
        height: SIZE,
        data,
    }
}
