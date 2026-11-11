//! Modèle du menu de l'icône de la barre d'état (clic droit) — la part
//! testable du tray.
//!
//! Le menu affiché est dessiné par l'hôte de la zone de notification
//! (Cinnamon, via DBus/StatusNotifierItem) : ses éléments ne vivent
//! que de DBus et restent vérifiables **par inspection de ce modèle**
//! (cf. `docs/RUST-EGUI.md`). Le service lui-même (`tray.rs`, binaire)
//! n'est que la colle qui expose ce modèle à l'hôte OS — aucune
//! décision dedans : pour chaque entrée, libellé du glossaire, geste
//! émis à l'activation et nom d'icône freedesktop vivent ici. L'hôte
//! OS reste seul responsable du rendu (dessin des icônes, police,
//! disposition du menu).

use crate::{labels, message::Message};

/// Icône freedesktop standard « configurer » (UC-04, extension 1a :
/// convention plateforme d'abord, cf. `docs/UX-GENERIC.md`).
const CONFIG_MENU_ICON: &str = "preferences-system";

/// Icône freedesktop standard « quitter » (UC-05, étape 2 : « un icône
/// parlant »).
const QUIT_MENU_ICON: &str = "application-exit";

/// Une entrée du menu de l'icône de la barre d'état.
pub struct TrayMenuEntry {
    /// Libellé affiché — terme du glossaire (module `labels`).
    pub label: &'static str,
    /// Message émis quand l'utilisateur active l'entrée (producteur :
    /// tray — l'hôte OS joue ce geste au point producteur de Messages).
    pub message: Message,
    /// Nom d'icône freedesktop affiché par l'hôte de la barre d'état
    /// (le rendu lui appartient, le choix du nom est ici).
    pub icon: &'static str,
}

/// Le menu du clic droit, dans l'ordre d'affichage de l'hôte.
///
/// - « Configurer... » : UC-04, extension 1a, étape 2 ;
/// - « Quitter Soporifique » : UC-05, étape 2.
pub fn menu() -> Vec<TrayMenuEntry> {
    vec![
        TrayMenuEntry {
            label: labels::TRAY_MENU_CONFIGURE,
            message: Message::ConfigOpenRequested,
            icon: CONFIG_MENU_ICON,
        },
        TrayMenuEntry {
            label: labels::TRAY_MENU_QUIT,
            message: Message::QuitRequested,
            icon: QUIT_MENU_ICON,
        },
    ]
}

#[cfg(test)]
mod tests {
    use super::*;

    /// UC-05, étape 2 : chaque entrée du menu porte « un icône
    /// parlant » — nom d'icône présent, libellé du glossaire présent.
    #[test]
    fn every_entry_carries_a_speaking_icon_and_label() {
        let entries = menu();
        assert!(
            entries.len() >= 2,
            "le menu affiche au moins configurer et quitter"
        );
        for entry in &entries {
            assert!(
                !entry.label.is_empty(),
                "chaque entrée porte un libellé du glossaire"
            );
            assert!(
                !entry.icon.is_empty(),
                "chaque entrée porte un nom d'icône freedesktop (icône parlante, UC-05 étape 2)"
            );
        }
    }
}
