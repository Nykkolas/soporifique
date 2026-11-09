//! Libellés UI (français) : termes du glossaire (`docs/GLOSSAIRE.md`).
//! Un seul lieu pour être relus et ciblés par les tests (cf.
//! `docs/RUST-EGUI.md`) — jamais de texte français disséminé dans `view`.

/// Entrée « Quitter » du menu de la barre d'état système (UC-05, étape 2).
pub const TRAY_MENU_QUIT: &str = "Quitter Soporifique";

/// Texte de la demande de confirmation de l'arrêt (UC-05, étape 4).
pub const QUIT_CONFIRMATION_TEXT: &str =
    "Voulez-vous vraiment quitter Soporifique ? La surveillance de la batterie sera arrêtée.";

/// Bouton de confirmation de l'arrêt (UC-05, étape 5).
pub const QUIT_CONFIRM_BUTTON: &str = "Oui";

/// Bouton d'annulation de l'arrêt (UC-05, extension 5a).
pub const QUIT_CANCEL_BUTTON: &str = "Non";

/// Nom de l'application : titre de la fenêtre et de l'icône de barre
/// d'état (nom propre, hors glossaire).
pub const APP_TITLE: &str = "Soporifique";
