use crate::{
    command::Command,
    message::Message,
    model::{ConfigWindow, Model, QuitRequest},
    store::SavedConfig,
};

/// Cœur TEA : reçoit un [`Message`], retourne le modèle mis à jour et les
/// [`Command`] à exécuter par le harnais. La garde des actions vit ici :
/// un message ne s'applique qu'à son état (cf. `docs/RUST-TEA.md`).
pub fn update(model: Model, message: Message) -> (Model, Vec<Command>) {
    match message {
        Message::QuitRequested => ask_quit(model),
        Message::QuitConfirmed => resolve_quit(model, true),
        Message::QuitCancelled => resolve_quit(model, false),
        Message::ConfigOpenRequested => open_config(model),
        Message::TestParamChanged(value) => change_test_param(model, value),
        Message::ConfigSaveRequested => request_save_config(model),
        Message::ConfigSaved => resolve_save_config(model, None),
        Message::ConfigSaveFailed(error) => resolve_save_config(model, Some(error)),
        Message::ConfigCancelled => cancel_config(model),
    }
}

/// Étape 3 → 4 : la demande de confirmation s'ouvre — le reste du
/// modèle est conservé (mise à jour fonctionnelle, cf. `docs/RUST-TEA.md`),
/// l'ouverture étant idempotente (déjà ouverte → déjà ouverte).
#[allow(clippy::needless_update)]
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
/// pas). Le reste du modèle est toujours conservé — un champ de
/// `Model` ne doit jamais être écrasé ici.
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

/// UC-04, étapes 1-1a → 2 : la fenêtre de configuration s'ouvre,
/// retenant les paramètres du moment pour « Annuler » (retour à la
/// valeur d'ouverture, extension 4a). Déjà ouverte (extensions 2a/2b) :
/// le modèle est inchangé, le geste devient une commande d'avant-plan
/// — la décision de fenêtre vit au runtime, qui la traduit.
#[allow(clippy::needless_update)]
fn open_config(model: Model) -> (Model, Vec<Command>) {
    if model.config_window.is_some() {
        return (model, vec![Command::RaiseWindow]);
    }
    (
        Model {
            config_window: Some(ConfigWindow {
                test_param_at_open: model.test_param,
                ..ConfigWindow::default()
            }),
            ..model
        },
        Vec::new(),
    )
}

/// UC-04, étape 3 : le paramètre d'essai change dans la fenêtre —
/// uniquement pendant qu'elle est ouverte et qu'aucune sauvegarde
/// n'est en attente de résultat (les actions sont désactivées pendant
/// la sauvegarde, la garde double la vue, cf. `docs/RUST-TEA.md`).
fn change_test_param(mut model: Model, value: bool) -> (Model, Vec<Command>) {
    let open_for_editing = model
        .config_window
        .as_ref()
        .is_some_and(|config| !config.saving);
    if !open_for_editing {
        return (model, Vec::new());
    }
    model.test_param = value;
    (model, Vec::new())
}

/// UC-04, étapes 4 → 5 : « Enregistrer » ordonne la persistance des
/// paramètres au moment du clic — la fenêtre attend le résultat
/// (`saving`) : elle ne se referme qu'au succès (étape 6), l'échec
/// garde la fenêtre ouverte avec l'erreur (garantie minimale UC-04).
/// Une double commande est ignorée : une sauvegarde à la fois.
fn request_save_config(model: Model) -> (Model, Vec<Command>) {
    match model.config_window.clone() {
        Some(config) if !config.saving => {
            let saved = SavedConfig {
                test_param: model.test_param,
            };
            (
                Model {
                    config_window: Some(ConfigWindow {
                        saving: true,
                        error: None,
                        ..config
                    }),
                    ..model
                },
                vec![Command::SaveConfig { saved }],
            )
        }
        _ => (model, Vec::new()),
    }
}

/// UC-04, étape 5 → 6 : le harnais a rapporté le résultat de la
/// persistance. Succès : la fenêtre se referme (étape 6). Échec : la
/// fenêtre reste ouverte, l'erreur s'affiche, les actions reprennent.
/// Hors sauvegarde en cours, le message est ignoré.
#[allow(clippy::needless_update)]
fn resolve_save_config(model: Model, error: Option<String>) -> (Model, Vec<Command>) {
    match model.config_window.clone() {
        Some(config) if config.saving => match error {
            None => (
                Model {
                    config_window: None,
                    ..model
                },
                Vec::new(),
            ),
            Some(error) => (
                Model {
                    config_window: Some(ConfigWindow {
                        saving: false,
                        error: Some(error),
                        ..config
                    }),
                    ..model
                },
                Vec::new(),
            ),
        },
        _ => (model, Vec::new()),
    }
}

/// UC-04, extension 4a : « Annuler » referme la fenêtre SANS enregistrer
/// — les paramètres retournent à leurs valeurs d'ouverture. Hors
/// fenêtre ouverte, le message est ignoré.
fn cancel_config(model: Model) -> (Model, Vec<Command>) {
    match model.config_window.as_ref() {
        Some(config) => (
            Model {
                config_window: None,
                test_param: config.test_param_at_open,
                ..model
            },
            Vec::new(),
        ),
        None => (model, Vec::new()),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn quit_requested_opens_the_confirmation() {
        let model = Model::default();
        let (model, commands) = update(model, Message::QuitRequested);
        assert!(
            model.quit_request.is_some(),
            "la demande de confirmation doit s'ouvrir"
        );
        assert!(
            commands.is_empty(),
            "aucun arrêt n'est ordonné sans confirmation"
        );
    }

    #[test]
    fn quit_confirmed_orders_exit_only_when_asked() {
        let (model, _) = update(Model::default(), Message::QuitRequested);
        let (model, commands) = update(model, Message::QuitConfirmed);
        assert!(
            model.quit_request.is_none(),
            "la demande se referme après confirmation"
        );
        assert_eq!(
            commands,
            vec![Command::ExitApplication],
            "l'arrêt doit être ordonné"
        );
    }

    #[test]
    fn quit_confirmed_without_request_is_ignored() {
        let (model, commands) = update(Model::default(), Message::QuitConfirmed);
        assert!(model.quit_request.is_none());
        assert!(
            commands.is_empty(),
            "aucun arrêt sans demande ouverte : garde d'état"
        );
    }

    #[test]
    fn quit_cancelled_closes_the_request_without_exit() {
        let (model, _) = update(Model::default(), Message::QuitRequested);
        let (model, commands) = update(model, Message::QuitCancelled);
        assert!(
            model.quit_request.is_none(),
            "« Non » referme la demande (extension 5a)"
        );
        assert!(commands.is_empty(), "« Non » n'ordonne jamais l'arrêt");
    }

    #[test]
    fn quit_cancelled_without_request_is_ignored() {
        let (model, commands) = update(Model::default(), Message::QuitCancelled);
        assert!(model.quit_request.is_none());
        assert!(commands.is_empty());
    }

    #[test]
    fn default_model_boots_with_params_unset() {
        let model = Model::default();
        assert!(
            !model.test_param,
            "premier lancement : paramètres par défaut"
        );
        assert!(model.quit_request.is_none() && model.config_window.is_none());
    }

    #[test]
    fn model_comes_from_saved_config() {
        let model = Model::from_saved(&SavedConfig { test_param: true });
        assert!(
            model.test_param,
            "le modèle boote depuis les paramètres persistés"
        );
    }

    #[test]
    fn config_open_opens_the_window_and_remembers_the_params() {
        let model = Model::from_saved(&SavedConfig { test_param: true });
        let (model, commands) = update(model, Message::ConfigOpenRequested);
        assert!(
            model.config_window.is_some(),
            "la fenêtre de configuration doit s'ouvrir"
        );
        assert_eq!(
            model.config_window.map(|config| config.test_param_at_open),
            Some(true),
            "l'ouverture retient les paramètres du moment pour « Annuler »"
        );
        assert!(commands.is_empty(), "aucun effet n'est ordonné en ouvrant");
    }

    #[test]
    fn config_open_is_idempotent() {
        let (mut model, commands) = update(Model::default(), Message::ConfigOpenRequested);
        assert!(
            commands.is_empty(),
            "l'ouverture elle-même n'ordonne rien : le modèle est la vérité"
        );
        model.test_param = true; // édition pendant que la fenêtre est ouverte
        let (model, commands) = update(model, Message::ConfigOpenRequested);
        assert!(
            model.config_window.is_some() && model.test_param,
            "réouvrir une fenêtre déjà ouverte ne doit rien changer au modèle \
             (extension 2a)"
        );
        assert_eq!(
            commands,
            vec![Command::RaiseWindow],
            "le geste sur une fenêtre déjà ouverte devient une commande \
             d'avant-plan (extensions 2a/2b)"
        );
    }

    #[test]
    fn test_param_changed_applies_while_config_window_is_open() {
        let (model, _) = update(Model::default(), Message::ConfigOpenRequested);
        let (model, commands) = update(model, Message::TestParamChanged(true));
        assert!(model.test_param, "cocher la case change le paramètre");
        assert!(commands.is_empty());
    }

    #[test]
    fn test_param_changed_without_window_is_ignored() {
        let (model, commands) = update(Model::default(), Message::TestParamChanged(true));
        assert!(!model.test_param, "garde d'état");
        assert!(commands.is_empty());
    }

    #[test]
    fn save_requested_orders_persist_and_waits_for_the_result() {
        let (model, _) = update(Model::default(), Message::ConfigOpenRequested);
        let (model, _) = update(model, Message::TestParamChanged(true));
        let (model, commands) = update(model, Message::ConfigSaveRequested);
        assert!(
            model
                .config_window
                .as_ref()
                .is_some_and(|config| config.saving),
            "après « Enregistrer » la fenêtre attend le résultat (étapes 4→5)"
        );
        assert_eq!(
            commands,
            vec![Command::SaveConfig {
                saved: SavedConfig { test_param: true }
            }],
            "l'ordre de persistance part vers le harnais avec les valeurs au clic"
        );
    }

    #[test]
    fn save_requested_twice_orders_only_one_save() {
        let (model, _) = update(Model::default(), Message::ConfigOpenRequested);
        let (model, _) = update(model, Message::ConfigSaveRequested);
        let (model, commands) = update(model, Message::ConfigSaveRequested);
        assert!(
            commands.is_empty(),
            "une sauvegarde à la fois : la double demande est ignorée"
        );
        assert!(
            model
                .config_window
                .as_ref()
                .is_some_and(|config| config.saving),
            "la première sauvegarde attend toujours son résultat"
        );
    }

    #[test]
    fn save_requested_without_window_is_ignored() {
        let (model, commands) = update(Model::default(), Message::ConfigSaveRequested);
        assert!(model.config_window.is_none());
        assert!(
            commands.is_empty(),
            "garde d'état : rien à persister fenêtre fermée"
        );
    }

    #[test]
    fn save_success_closes_the_window_keeping_the_params() {
        let (model, _) = update(Model::default(), Message::ConfigOpenRequested);
        let (model, _) = update(model, Message::TestParamChanged(true));
        let (model, _) = update(model, Message::ConfigSaveRequested);
        let (model, commands) = update(model, Message::ConfigSaved);
        assert!(
            model.config_window.is_none(),
            "persistance réussie : la fenêtre se referme (étape 6)"
        );
        assert!(
            model.test_param,
            "les paramètres du moment du clic sont conservés (étape 5)"
        );
        assert!(commands.is_empty());
    }

    #[test]
    fn save_failure_keeps_the_window_open_with_the_error() {
        let (model, _) = update(Model::default(), Message::ConfigOpenRequested);
        let (model, _) = update(model, Message::ConfigSaveRequested);
        let (model, commands) = update(model, Message::ConfigSaveFailed("disk full".into()));
        let window = model
            .config_window
            .expect("la fenêtre reste ouverte en cas d'échec");
        assert!(!window.saving, "les actions reprennent");
        assert_eq!(
            window.error.as_deref(),
            Some("disk full"),
            "l'erreur remonte à la fenêtre (garantie minimale UC-04 : erreur claire)"
        );
        assert!(commands.is_empty());
    }

    #[test]
    fn save_result_without_pending_save_is_ignored() {
        let (model, commands) = update(Model::default(), Message::ConfigSaved);
        assert!(model.config_window.is_none());
        assert!(
            commands.is_empty(),
            "garde d'état : aucun résultat inattendu n'agit"
        );

        let (model, _) = update(Model::default(), Message::ConfigOpenRequested);
        let (model, commands) = update(model, Message::ConfigSaved);
        assert!(
            model.config_window.is_some(),
            "un résultat sans sauvegarde en cours ne referme pas la fenêtre"
        );
        assert!(commands.is_empty());
    }

    #[test]
    fn config_cancel_restores_the_params_from_opening() {
        let (model, _) = update(Model::default(), Message::ConfigOpenRequested);
        let (model, _) = update(model, Message::TestParamChanged(true));
        let (model, commands) = update(model, Message::ConfigCancelled);
        assert!(
            model.config_window.is_none(),
            "« Annuler » referme la fenêtre (extension 4a)"
        );
        assert!(
            !model.test_param,
            "« Annuler » referme sans enregistrer : retour à la valeur d'ouverture"
        );
        assert!(commands.is_empty());
    }

    #[test]
    fn config_cancel_without_window_is_ignored() {
        let (model, commands) = update(Model::default(), Message::ConfigCancelled);
        assert!(model.config_window.is_none());
        assert!(!model.test_param, "garde d'état");
        assert!(commands.is_empty());
    }
}
