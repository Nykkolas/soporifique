/// Messages : un `Message` par geste, tous producteurs confondus
/// (UI egui, tray ksni, timer de polling) — cf. `docs/RUST-TEA.md`.
pub enum Message {
    /// Entrée « Quitter Soporifique » activée dans le menu de la barre
    /// d'état système (producteur : tray, UC-05 étape 3).
    QuitRequested,
    /// L'utilisateur confirme l'arrêt (« Oui ») dans la demande de
    /// confirmation (producteur : UI egui, UC-05 étape 5).
    QuitConfirmed,
    /// L'utilisateur annule l'arrêt (« Non ») dans la demande de
    /// confirmation (producteur : UI egui, UC-05 extension 5a).
    QuitCancelled,
}
