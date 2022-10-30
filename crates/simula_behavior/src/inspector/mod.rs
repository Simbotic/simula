use bevy::prelude::*;
use bevy_inspector_egui::{Inspectable, InspectorPlugin};
pub use inspector::{BehaviorInspector, BehaviorInspectorAttributes};
pub use node::{BehaviorInspectorNode, BehaviorInspectorNodeAttributes};

pub mod inspector;
pub mod node;

pub struct BehaviorInspectorPlugin;

impl Plugin for BehaviorInspectorPlugin {
    fn build(&self, app: &mut App) {
        app.insert_resource(BehaviorInspectorAttributes {
            name: "Behavior".to_string(),
            description: "Behavior".to_string(),
        })
        .add_startup_system(setup)
        .add_plugin(InspectorPlugin::<BehaviorInspector>::new());
    }
}

fn setup() {}
