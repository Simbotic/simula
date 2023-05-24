use bevy::prelude::*;
pub use inspector::{BehaviorInspector, BehaviorInspectorAttributes, behavior_inspector_ui};
pub use node::{BehaviorInspectorNode, BehaviorInspectorNodeAttributes};

pub mod inspector;
pub mod node;
pub mod graph;

pub struct BehaviorInspectorPlugin;

impl Plugin for BehaviorInspectorPlugin {
    fn build(&self, app: &mut App) {
        app.insert_resource(BehaviorInspectorAttributes)
            .add_startup_system(setup)
            .insert_resource(BehaviorInspector::default());
            // .add_system(behavior_inspector_ui);
    }
}

fn setup() {}
