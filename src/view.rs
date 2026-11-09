use crate::model::Model;

/// Rendu du modèle. Ne lit que le modèle, n'exécute rien.
/// Point d'entrée temporaire textuel (l'UI egui arrivera avec le premier UC).
pub fn view(model: &Model) {
    println!("Hello, soporifique ! {:?}", model);
}
