//! Persistance des paramètres entre les lancements (UC-04, étape 5 :
//! « L'application sauvegarde les paramètres ») — effet extérieur par
//! trait injecté : le domaine ne connaît ni le disque ni les chemins,
//! seul le harnais fournit le stockage (cf. `docs/RUST-TEA.md`).
//!
//! La gestion de la sauvegarde vit dans ce module, partagée par tous
//! les UC paramètres : un UC paramètre (le seuil avec l'UC-01) apporte
//! son libellé, son widget et sa validation — jamais sa propre
//! mécanique d'écriture.
//!
//! Format : une ligne `clé=valeur` par paramètre (choix d'implémentation
//! figé par la spec test UC-04). Évolutif : ajouter un paramètre est
//! rétro-compatible (clé absente = valeur par défaut) ; le jour où un
//! paramètre exigera structure, listes ou commentaires, la migration
//! (ex. TOML) se borne à ce module — le format ne fuit nulle part
//! ailleurs.
//!
//! Parse strict : ligne malformée, clé inconnue ou doublon = erreur
//! explicite, jamais de valeur devinée (garantie minimale UC-04 : un
//! problème signalé clairement).

use std::{fs, path::PathBuf};

use crate::model::Model;

/// Les paramètres persistés entre les lancements de l'application
/// (UC-04, étape 5) — la projection sur disque du modèle.
///
/// Aujourd'hui uniquement le paramètre d'essai (cf. `Model::test_param`).
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct SavedConfig {
    /// Paramètre d'essai (UC-04) : case à cocher sans valeur domaine,
    /// seul but permettre à l'utilisateur de tester lui-même la chaîne
    /// enregistrer/relire ; remplacée par les vrais paramètres à
    /// validation de leurs UC.
    pub test_param: bool,
}

/// Effet extérieur : persister les paramètres — injecté par le harnais
/// (`main.rs` : fichier `config` dans XDG_CONFIG_HOME ; tests : fichier
/// temporaire). Toute erreur remonte en [`String`], message anglais
/// destiné au journal à venir (le préfixe français à l'écran vit dans
/// `labels`).
pub trait ConfigStore {
    /// Charge les paramètres : `None` au premier lancement — aucun
    /// fichier n'existe encore (les valeurs par défaut s'appliquent).
    fn load_saved(&self) -> Result<Option<SavedConfig>, String>;
    /// Enregistre les paramètres sur le disque (UC-04, étape 5).
    fn save_saved(&mut self, saved: &SavedConfig) -> Result<(), String>;
}

/// Persistance dans un fichier : une ligne `clé=valeur` par paramètre
/// (rendu/parse purs ci-dessous, testables sans disque).
pub struct FileConfigStore {
    /// Chemin du fichier de configuration (« config » sous le répertoire
    /// de l'application — l'extension est superflue, le format est un
    /// détail privé du store).
    path: PathBuf,
}

impl FileConfigStore {
    pub fn new(path: impl Into<PathBuf>) -> Self {
        Self { path: path.into() }
    }
}

impl ConfigStore for FileConfigStore {
    fn load_saved(&self) -> Result<Option<SavedConfig>, String> {
        if !self.path.exists() {
            return Ok(None);
        }
        let contents = fs::read_to_string(&self.path).map_err(|error| {
            format!(
                "Unable to read config file {}: {error}",
                self.path.display()
            )
        })?;
        parse_saved(&contents).map(Some)
    }

    fn save_saved(&mut self, saved: &SavedConfig) -> Result<(), String> {
        if let Some(dir) = self.path.parent() {
            fs::create_dir_all(dir).map_err(|error| {
                format!("Unable to create config dir {}: {error}", dir.display())
            })?;
        }
        fs::write(&self.path, render_saved(saved)).map_err(|error| {
            format!(
                "Unable to write config file {}: {error}",
                self.path.display()
            )
        })
    }
}

/// Rendu du fichier : une ligne `clé=valeur` par paramètre, dans un
/// ordre stable (la spec test UC-04 fige le contenu exact).
pub fn render_saved(saved: &SavedConfig) -> String {
    format!("test_param={}\n", saved.test_param)
}

/// Parse du fichier : strict — ligne malformée, clé inconnue ou
/// doublon = erreur explicite, jamais de valeur devinée. Une clé
/// absente garde sa valeur par défaut.
pub fn parse_saved(contents: &str) -> Result<SavedConfig, String> {
    // Paramètres vus : `Option` par paramètre → un doublon est détecté,
    // une clé absente garde sa valeur par défaut (`unwrap_or`).
    let mut test_param: Option<bool> = None;
    for (index, line) in contents.lines().filter(|line| !line.is_empty()).enumerate() {
        let lineno = index + 1;
        let (key, value) = line.split_once('=').ok_or_else(|| {
            format!("Malformed config line {lineno}: expected `key=value`, got `{line}`")
        })?;
        match key {
            "test_param" => {
                if test_param.is_some() {
                    return Err(format!(
                        "Duplicate config key `test_param` at line {lineno}"
                    ));
                }
                test_param = Some(parse_bool(&format!("test_param (line {lineno})"), value)?);
            }
            unknown => return Err(format!("Unknown config key `{unknown}` at line {lineno}")),
        }
    }
    Ok(SavedConfig {
        test_param: test_param.unwrap_or(false),
    })
}

/// Parse booléen strict : seulement `true` ou `false`.
fn parse_bool(what: &str, value: &str) -> Result<bool, String> {
    match value {
        "true" => Ok(true),
        "false" => Ok(false),
        _ => Err(format!(
            "Invalid boolean for {what}: expected `true` or `false`, got `{value}`"
        )),
    }
}

/// Lancement en paramètres : le modèle initial de l'application, booté
/// depuis le stockage — logique de lancement pure, partagée par
/// `main.rs` et les spec tests (aucune duplication du câblage).
///
/// Échec de lecture : la garantie minimale de l'UC-04 promet une erreur
/// clairement signalée — le journal vivra avec son UC ; pour l'instant
/// l'erreur passe au harnais (retournée), l'application démarre sur
/// les valeurs par défaut.
pub fn boot_model(store: &dyn ConfigStore) -> (Model, Option<String>) {
    match store.load_saved() {
        Ok(Some(saved)) => (Model::from_saved(&saved), None),
        Ok(None) => (Model::default(), None),
        Err(warning) => (Model::default(), Some(warning)),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn file_store_roundtrips_a_saved_config() {
        let path = std::env::temp_dir().join(format!(
            "soporifique-store-roundtrip-{}.config",
            std::process::id()
        ));
        if path.exists() {
            std::fs::remove_file(&path).expect("nettoyage avant test");
        }
        let mut store = FileConfigStore::new(path.clone());
        assert_eq!(
            store
                .load_saved()
                .expect("premier lancement sans fichier : lecture saine"),
            None,
            "aucun fichier = aucun paramètre : défauts"
        );
        store
            .save_saved(&SavedConfig { test_param: true })
            .expect("l'écriture doit réussir");
        assert_eq!(
            store.load_saved().expect("relecture du fichier sauvé"),
            Some(SavedConfig { test_param: true }),
            "aller-retour disque : ce qui est sauvé doit être relu"
        );
        std::fs::remove_file(&path).expect("nettoyage après test");
    }

    #[test]
    fn file_store_renders_the_exact_file_contents() {
        assert_eq!(
            render_saved(&SavedConfig { test_param: false }),
            "test_param=false\n"
        );
        assert_eq!(
            render_saved(&SavedConfig { test_param: true }),
            "test_param=true\n"
        );
    }

    #[test]
    fn parse_accepts_strictly_only_known_values() {
        let saved = parse_saved("test_param=true").expect("valeur valide");
        assert!(saved.test_param);
        let saved = parse_saved("test_param=false").expect("valeur valide");
        assert!(!saved.test_param);
        assert!(
            parse_saved("").is_ok(),
            "fichier vide : aucun paramètre, défauts"
        );

        assert!(
            parse_saved("test_param=yes").is_err(),
            "booléen malformé : erreur explicite"
        );
        assert!(
            parse_saved("test_param").is_err(),
            "ligne sans `=` : erreur explicite"
        );
        assert!(
            parse_saved("seuil=10").is_err(),
            "clé inconnue : erreur explicite (les paramètres futurs s'ajouteront ici)"
        );
        assert!(
            parse_saved("test_param=true\ntest_param=false").is_err(),
            "doublon de clé : erreur explicite, jamais de valeur devinée"
        );
    }

    #[test]
    fn boot_model_applies_defaults_then_saved_config() {
        struct MemoryStore(Option<SavedConfig>);
        impl ConfigStore for MemoryStore {
            fn load_saved(&self) -> Result<Option<SavedConfig>, String> {
                Ok(self.0.clone())
            }
            fn save_saved(&mut self, _saved: &SavedConfig) -> Result<(), String> {
                Ok(())
            }
        }

        let (model, warning) = boot_model(&MemoryStore(None));
        assert!(!model.test_param, "premier lancement : défauts");
        assert!(warning.is_none());

        let (model, warning) = boot_model(&MemoryStore(Some(SavedConfig { test_param: true })));
        assert!(model.test_param, "paramètre sauvé relu au démarrage");
        assert!(warning.is_none());

        struct BrokenStore;
        impl ConfigStore for BrokenStore {
            fn load_saved(&self) -> Result<Option<SavedConfig>, String> {
                Err("disk on fire".into())
            }
            fn save_saved(&mut self, _saved: &SavedConfig) -> Result<(), String> {
                Ok(())
            }
        }
        let (model, warning) = boot_model(&BrokenStore);
        assert!(
            !model.test_param,
            "échec de lecture : défauts, pas de plantage"
        );
        assert_eq!(
            warning.as_deref(),
            Some("disk on fire"),
            "l'échec remonte au harnais — le journal vivra avec son UC"
        );
    }
}
