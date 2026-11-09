//! Soporifique : surveille le niveau de batterie et met le portable en
//! hibernation au franchissement d'un seuil configurable.
//!
//! Pattern TEA impose par la structure (voir docs/RUST-TEA.md).
//! Squelette : compilation et.* rien d'autre*.

mod message;
mod model;
mod update;
mod view;

fn main() {
    let model = model::Model;
    view::view(&model);
}
