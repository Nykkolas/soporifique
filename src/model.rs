//! État de l'application, immuable, mis à jour uniquement par `update`.

use crate::store::SavedConfig;

/// La demande de confirmation d'arrêt affichée à l'utilisateur
/// (UC-05, étapes 4-5). Sans donnée pour l'instant.
#[derive(Debug, Clone)]
pub struct QuitRequest;

/// La fenêtre de configuration ouverte à l'utilisateur (UC-04,
/// étapes 2 → 6 / extension 4a).
#[derive(Debug, Clone, Default)]
pub struct ConfigWindow {
    /// Paramètre d'essai tel qu'à l'ouverture de la fenêtre : ce que
    /// « Annuler » restaure en refermant sans enregistrer (UC-04,
    /// extension 4a : l'état retourne à la valeur d'ouverture).
    pub test_param_at_open: bool,
    /// Sauvegarde en cours (UC-04, étapes 4 → 5) : le clic
    /// « Enregistrer » a ordonné la persistance, la fenêtre attend le
    /// résultat, actions désactivées.
    pub saving: bool,
    /// Erreur de sauvegarde à afficher dans la fenêtre (garantie
    /// minimale UC-04 : une erreur claire ; le journal détaillé vivra
    /// avec son UC).
    pub error: Option<String>,
}

/// Le modèle : son sous-état vivant est la présence ou non d'une
/// demande à l'écran (confirmation d'arrêt, fenêtre de configuration).
#[derive(Debug, Clone, Default)]
pub struct Model {
    /// Demande de confirmation d'arrêt : `Some` tant que la fenêtre est
    /// affichée (UC-05, étape 4 → extension 5a).
    pub quit_request: Option<QuitRequest>,
    /// Fenêtre de configuration : `Some` tant qu'elle est affichée
    /// (UC-04, étape 2 → étape 6).
    pub config_window: Option<ConfigWindow>,
    /// Paramètre d'essai (UC-04) : case à cocher sans valeur domaine,
    /// seul but permettre à l'utilisateur de tester lui-même la chaîne
    /// enregistrer/relire ; remplacée par les vrais paramètres à
    /// validation de leurs UC (le seuil avec l'UC-01). Persisté par le
    /// store entre les lancements — traitée comme un vrai paramètre
    /// par tout le reste du code.
    pub test_param: bool,
}

impl Model {
    /// Modèle booté depuis les paramètres persistés au lancement
    /// précédent (cf. `store::boot_model`).
    pub fn from_saved(saved: &SavedConfig) -> Self {
        Self {
            test_param: saved.test_param,
            ..Self::default()
        }
    }
}
