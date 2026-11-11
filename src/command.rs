//! Command = effet différé : `update` retourne des [`Command`] (valeurs),
//! seul le harnais les exécute (cf. `docs/RUST-TEA.md`).

use crate::store::SavedConfig;

/// Effets que le cœur peut ordonner au harnais.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Command {
    /// Ordonner au harnais d'arrêter l'application
    /// (UC-05, étape 6 : « L'application se ferme »).
    ExitApplication,
    /// Ramener la fenêtre de configuration à l'avant-plan (UC-04,
    /// extensions 2a/2b : le geste « ouvrir la config » arrive alors
    /// qu'elle est déjà à l'écran — le modèle est inchangé, seule la
    /// fenêtre est concernée, décision du shell traduite en
    /// [`WindowEffect::Raise`](crate::runtime::WindowEffect) par le
    /// runtime).
    RaiseWindow,
    /// Persister les paramètres sur disque (UC-04, étape 5) : les
    /// valeurs au moment du clic « Enregistrer ». Le harnais exécute
    /// puis rapporte le résultat — `ConfigSaved` / `ConfigSaveFailed`
    /// (chaque Command rapporte, cf. `docs/RUST-TEA.md`).
    SaveConfig { saved: SavedConfig },
}
