use crate::{labels, message::Message, model::Model};

/// Marge intérieure de la demande de confirmation.
const MARGIN: i8 = 14;

/// Rendu du modèle : ne lit que le modèle, rend l'UI et émet ses
/// [`Message`] dans la boîte d'émission — n'exécute rien d'autre
/// (cf. `docs/RUST-EGUI.md`).
///
/// Un seul contenu existe aujourd'hui : la demande de confirmation
/// d'arrêt (UC-05) — la fenêtre de configuration (UC-04) s'ajoutera ici.
///
/// La fenêtre qui porte ce rendu est gérée par le harnais (fenêtre
/// racine montrée uniquement pendant la demande — cf. `main.rs`) ;
/// en headless (egui_kittest, spec test), la même vue vit dans la passe
/// unique. Un seul code, deux supports.
pub fn view(model: &Model, ui: &mut egui::Ui, outbox: &mut Vec<Message>) {
    if model.quit_request.is_some() {
        confirmation(ui, outbox);
    }
}

/// Demande de confirmation d'arrêt (UC-05, étapes 4-5 / extension 5a).
///
/// Convention des dialogues de confirmation (cf. `docs/UX-GENERIC.md`) :
/// libellé explicatif ; actions opposées (quitter / annuler) alignées à
/// droite, l'action destructive séparée de son annulation.
fn confirmation(ui: &mut egui::Ui, outbox: &mut Vec<Message>) {
    egui::Frame::default()
        .inner_margin(egui::Margin::same(MARGIN))
        .show(ui, |ui| {
            ui.label(labels::QUIT_CONFIRMATION_TEXT);
            ui.add_space(f32::from(MARGIN));
            ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                if ui.button(labels::QUIT_CONFIRM_BUTTON).clicked() {
                    outbox.push(Message::QuitConfirmed);
                }
                if ui.button(labels::QUIT_CANCEL_BUTTON).clicked() {
                    outbox.push(Message::QuitCancelled);
                }
            });
        });
}
