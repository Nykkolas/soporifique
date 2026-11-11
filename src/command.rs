//! Command = effet différé : `update` retourne des [`Command`] (valeurs),
//! seul le harnais les exécute (cf. `docs/RUST-TEA.md`).

use crate::store::SavedConfig;

/// Effets que le cœur peut ordonner au harnais.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Command {
    /// Ordonner au harnais d'arrêter l'application
    /// (UC-05, étape 6 : « L'application se ferme »).
    ExitApplication,
    /// Persister les paramètres sur disque (UC-04, étape 5) : les
    /// valeurs au moment du clic « Enregistrer ». Le harnais exécute
    /// puis rapporte le résultat — `ConfigSaved` / `ConfigSaveFailed`
    /// (chaque Command rapporte, cf. `docs/RUST-TEA.md`).
    SaveConfig { saved: SavedConfig },
}
