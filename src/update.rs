use crate::{command::Command, message::Message, model::{Model, QuitRequest}};

/// Cœur TEA : reçoit un [`Message`], retourne le modèle mis à jour et les
/// [`Command`] à exécuter par le harnais. La garde des actions vit ici :
/// un message ne s'applique qu'à son état (cf. `docs/RUST-TEA.md`).
pub fn update(model: Model, message: Message) -> (Model, Vec<Command>) {
    match message {
        Message::QuitRequested => ask_quit(model),
        Message::QuitConfirmed => resolve_quit(model, true),
        Message::QuitCancelled => resolve_quit(model, false),
    }
}

/// Étape 3 → 4 : la demande de confirmation s'ouvre — le reste du
/// modèle est conservé (mise à jour fonctionnelle, cf. `docs/RUST-TEA.md`),
/// l'ouverture étant idempotente (déjà ouverte → déjà ouverte).
/// Mise à jour fonctionnelle anticipée : `..model` est volontairement
/// redondant tant que `Model` n'a qu'un champ (rustc parle de variable
/// inutilisée, clippy de « needless update ») — il préserve les champs
/// futurs (fenêtre UC-04, seuil UC-01), cf. audit 2026-11-09.
#[allow(unused_variables, clippy::needless_update)]
fn ask_quit(model: Model) -> (Model, Vec<Command>) {
    (
        Model {
            quit_request: Some(QuitRequest),
            ..model
        },
        Vec::new(),
    )
}

/// Étapes 5 / 5a : « Oui » ordonne l'arrêt et referme la demande ;
/// « Non » la referme sans rien ordonner. Hors demande ouverte, le
/// message est ignoré (un clic sous une fenêtre disparue ne quitte
/// pas). Le reste du modèle est toujours conservé — un futur champ
/// (fenêtre UC-04, seuil UC-01) ne doit jamais être écrasé ici.
/// La garde des actions vit ici : cf. `ask_quit` pour la mise à jour
/// fonctionnelle anticipée (`..model`), cf. audit 2026-11-09.
#[allow(clippy::needless_update)]
fn resolve_quit(model: Model, confirmed: bool) -> (Model, Vec<Command>) {
    let quit_requested = model.quit_request.is_some();
    match (quit_requested, confirmed) {
        (true, true) => (
            Model {
                quit_request: None,
                ..model
            },
            vec![Command::ExitApplication],
        ),
        (true, false) => (
            Model {
                quit_request: None,
                ..model
            },
            Vec::new(),
        ),
        (false, _) => (model, Vec::new()),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn quit_requested_opens_the_confirmation() {
        let model = Model::default();
        let (model, commands) = update(model, Message::QuitRequested);
        assert!(model.quit_request.is_some(), "la demande de confirmation doit s'ouvrir");
        assert!(commands.is_empty(), "aucun arrêt n'est ordonné sans confirmation");
    }

    #[test]
    fn quit_confirmed_orders_exit_only_when_asked() {
        let (model, _) = update(Model::default(), Message::QuitRequested);
        let (model, commands) = update(model, Message::QuitConfirmed);
        assert!(model.quit_request.is_none(), "la demande se referme après confirmation");
        assert_eq!(commands, vec![Command::ExitApplication], "l'arrêt doit être ordonné");
    }

    #[test]
    fn quit_confirmed_without_request_is_ignored() {
        let (model, commands) = update(Model::default(), Message::QuitConfirmed);
        assert!(model.quit_request.is_none());
        assert!(commands.is_empty(), "aucun arrêt sans demande ouverte : garde d'état");
    }

    #[test]
    fn quit_cancelled_closes_the_request_without_exit() {
        let (model, _) = update(Model::default(), Message::QuitRequested);
        let (model, commands) = update(model, Message::QuitCancelled);
        assert!(model.quit_request.is_none(), "« Non » referme la demande (extension 5a)");
        assert!(commands.is_empty(), "« Non » n'ordonne jamais l'arrêt");
    }

    #[test]
    fn quit_cancelled_without_request_is_ignored() {
        let (model, commands) = update(Model::default(), Message::QuitCancelled);
        assert!(model.quit_request.is_none());
        assert!(commands.is_empty());
    }
}
