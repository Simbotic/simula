use bevy::prelude::*;
use bevy_inspector_egui::{Inspectable, InspectorPlugin};

#[derive(Inspectable, Default)]
pub struct BehaviorInspector {
    pub behavior_root: Option<Entity>,
}

pub struct BehaviorInspectorPlugin;

impl Plugin for BehaviorInspectorPlugin {
    fn build(&self, app: &mut App) {
        app.add_startup_system(setup)
        .add_plugin(InspectorPlugin::<BehaviorInspector>::new());
    }
}

fn setup() {}
