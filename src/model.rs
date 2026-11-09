//! État de l'application, immuable, mis à jour uniquement par `update`.

/// La demande de confirmation d'arrêt affichée à l'utilisateur
/// (UC-05, étapes 4-5). Sans donnée pour l'instant.
#[derive(Debug, Clone)]
pub struct QuitRequest;

/// Le modèle : son sous-état vivant est la présence ou non de la
/// fenêtre de confirmation d'arrêt (UC-04 : la fenêtre de configuration
/// viendra s'ajouter ici sans casser le modèle).
#[derive(Debug, Clone, Default)]
pub struct Model {
    /// Demande de confirmation d'arrêt : `Some` tant que la fenêtre est
    /// affichée (UC-05, étape 4 → extension 5a).
    pub quit_request: Option<QuitRequest>,
}
