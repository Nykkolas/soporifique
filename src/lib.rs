//! Soporifique — cœur TEA de l'application.
//!
//! Pattern TEA imposé par la structure des fichiers (cf. `docs/RUST-TEA.md`) :
//! `main.rs` n'est que le harnais (câblage egui/ksni, exécution des
//! [`command::Command`]) ; toute la logique vit ici, pure et testable sans
//! écran ni batterie réelle.

pub mod command;
pub mod labels;
pub mod message;
pub mod model;
pub mod runtime;
pub mod update;
pub mod view;
