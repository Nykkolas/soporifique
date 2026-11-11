use crate::{labels, message::Message, model::Model};

/// Marge intérieure des fenêtres à la demande.
const MARGIN: i8 = 14;

/// Rendu du modèle : ne lit que le modèle, rend l'UI et émet ses
/// [`Message`] dans la boîte d'émission — n'exécute rien d'autre
/// (cf. `docs/RUST-EGUI.md`).
///
/// La fenêtre qui porte ce rendu est gérée par le harnais (fenêtre
/// unique montrée uniquement pendant une demande — cf. `main.rs`) ;
/// en headless (egui_kittest, spec test), la même vue vit dans la passe
/// unique. Un seul code, deux supports.
pub fn view(model: &Model, ui: &mut egui::Ui, outbox: &mut Vec<Message>) {
    if model.quit_request.is_some() {
        confirmation(ui, outbox);
    }
    if model.config_window.is_some() {
        configuration(model, ui, outbox);
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

/// Fenêtre de configuration (UC-04, étapes 2-6) : chaque paramètre
/// spécifié par les UC avec son widget de saisie, l'erreur éventuelle
/// de sauvegarde, puis les actions de la fenêtre (cf.
/// `docs/UX-GENERIC.md`) : « Enregistrer » (persiste et referme)
/// opposé à « Annuler » (referme sans enregistrer), alignés à droite.
///
/// Mode immédiat + TEA : chaque widget édite une copie locale et émet
/// un [`Message`] à chaque geste — l'état visible vit dans le modèle,
/// jamais dans egui (cf. `docs/RUST-EGUI.md`). Pendant la sauvegarde
/// (`saving`), la saisie et les actions sont désactivées : le résultat
/// de la persistance décide de la fermeture, pas le clic.
fn configuration(model: &Model, ui: &mut egui::Ui, outbox: &mut Vec<Message>) {
    // La fenêtre est ouverte (garde de `view`) ; les champs optionnels
    // par défaut évitent un clone du modèle entier.
    let window = model.config_window.clone().unwrap_or_default();
    egui::Frame::default()
        .inner_margin(egui::Margin::same(MARGIN))
        .show(ui, |ui| {
            // Paramètre d'essai (UC-04, étape 3) — remplacé par les
            // vrais paramètres à validation de leurs UC (cf. labels).
            let mut test_param = model.test_param;
            let checkbox = ui.add_enabled(
                !window.saving,
                egui::Checkbox::new(&mut test_param, labels::TEST_PARAM_LABEL),
            );
            if checkbox.changed() {
                outbox.push(Message::TestParamChanged(test_param));
            }

            // Garantie minimale UC-04 : l'échec de sauvegarde s'affiche
            // dans la fenêtre (le journal détaillé suivra avec son UC).
            if let Some(error) = &window.error {
                ui.label(format!("{}{}", labels::SAVE_FAILED_PREFIX, error));
            }

            ui.add_space(f32::from(MARGIN));
            ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                if ui
                    .add_enabled(!window.saving, egui::Button::new(labels::SAVE_BUTTON))
                    .clicked()
                {
                    outbox.push(Message::ConfigSaveRequested);
                }
                if ui
                    .add_enabled(!window.saving, egui::Button::new(labels::CANCEL_BUTTON))
                    .clicked()
                {
                    outbox.push(Message::ConfigCancelled);
                }
            });
        });
}
