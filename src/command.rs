//! Command = effet différé : `update` retourne des [`Command`] (valeurs),
//! seul le harnais les exécute (cf. `docs/RUST-TEA.md`).

/// Effets que le cœur peut ordonner au harnais.
///
/// Squelette UC-05 : seul l'arrêt de l'application est spécifié ;
/// la lecture de batterie et l'hibernation suivront avec les UC-01/02.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Command {
    /// Ordonner au harnais d'arrêter l'application
    /// (UC-05, étape 6 : « L'application se ferme »).
    ExitApplication,
}
