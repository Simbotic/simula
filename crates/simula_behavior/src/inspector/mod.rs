use bevy::prelude::*;
use bevy_inspector_egui::InspectorPlugin;
pub use inspector::{BehaviorInspector, BehaviorInspectorAttributes};
pub use node::{BehaviorInspectorNode, BehaviorInspectorNodeAttributes};

pub mod inspector;
pub mod node;

pub struct BehaviorInspectorPlugin;

impl Plugin for BehaviorInspectorPlugin {
    fn build(&self, app: &mut App) {
        app.insert_resource(BehaviorInspectorAttributes)
            .add_startup_system(setup)
            .add_plugin(InspectorPlugin::<BehaviorInspector>::new());
    }
}

fn setup() {}
