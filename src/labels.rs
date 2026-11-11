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

/// Titre de la fenêtre de configuration (UC-04, étape 2).
pub const CONFIG_WINDOW_TITLE: &str = "Soporifique — Configuration";

/// Libellé du paramètre d'essai dans la fenêtre de configuration
/// (UC-04) : case à cocher de test de la chaîne enregistrer/relire,
/// sans valeur domaine — remplacée par les vrais paramètres à
/// validation de leurs UC (le seuil avec l'UC-01).
pub const TEST_PARAM_LABEL: &str = "Paramètre d'essai";

/// Bouton « Enregistrer » de la fenêtre de configuration
/// (UC-04, étape 4 ; glossaire : « Enregistrer », `save`).
pub const SAVE_BUTTON: &str = "Enregistrer";

/// Bouton « Annuler » de la fenêtre de configuration
/// (UC-04, extension 4a ; glossaire : « Annuler », `cancel`).
pub const CANCEL_BUTTON: &str = "Annuler";

/// Préfixe français de l'erreur de sauvegarde affichée dans la
/// fenêtre de configuration (garantie minimale UC-04) : le détail
/// technique anglais du store suit (destiné au journal, à venir).
pub const SAVE_FAILED_PREFIX: &str = "Enregistrement impossible : ";

/// Entrée « Configurer » du menu de l'icône de la barre d'état
/// (UC-04, extension 1a, étape 2).
pub const TRAY_MENU_CONFIGURE: &str = "Configurer...";
