use crate::{message::Message, model::Model};

/// Cœur TEA : reçoit un Message, retourne le modèle mis à jour.
/// `match` exhaustif — vide pour l'instant, aucun geste spécifié.
///
/// Squelette : `dead_code`/`unused_variables` autorisés jusqu'au branchement
/// du harnais egui (premier UC) — à retirer alors.
#[allow(dead_code, unused_variables)]
pub fn update(model: Model, message: Message) -> Model {
    match message {}
}
