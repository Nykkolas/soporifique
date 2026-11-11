/// Messages : un `Message` par geste, tous producteurs confondus
/// (UI egui, tray ksni, timer de polling) — cf. `docs/RUST-TEA.md`.
#[derive(Clone)]
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
    /// Clic gauche sur l'icône — signal `Activate` — ou entrée
    /// « Configurer... » du menu de la barre d'état (producteur : tray,
    /// UC-04 étapes 1 et 1a).
    ConfigOpenRequested,
    /// L'utilisateur modifie le paramètre d'essai dans la fenêtre de
    /// configuration (producteur : UI egui, UC-04 étape 3).
    TestParamChanged(bool),
    /// L'utilisateur clique sur « Enregistrer » dans la fenêtre de
    /// configuration (producteur : UI egui, UC-04 étape 4 → 5).
    ConfigSaveRequested,
    /// Le harnais a persisté les paramètres sur disque (producteur :
    /// harnais exécutant la commande, UC-04 étape 5 → 6).
    ConfigSaved,
    /// Le harnais n'a pas pu persister les paramètres (producteur :
    /// harnais exécutant la commande ; garantie minimale UC-04 :
    /// l'erreur remonte à la fenêtre, le journal suivra avec son UC).
    ConfigSaveFailed(String),
    /// L'utilisateur referme la fenêtre de configuration sans
    /// enregistrer — bouton « Annuler » (UC-04, extension 4a) ou ✕ de
    /// la fenêtre (producteurs : UI egui, runtime).
    ConfigCancelled,
}
