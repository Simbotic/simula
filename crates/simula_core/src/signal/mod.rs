use bevy::prelude::*;

pub mod controller;
pub mod generator;

pub use controller::SignalController;
pub use generator::{SignalFunction, SignalGenerator};

pub struct SignalPlugin;

impl Plugin for SignalPlugin {
    fn build(&self, app: &mut App) {
        app.register_type::<SignalFunction>();
    }
}
