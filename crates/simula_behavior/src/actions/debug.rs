use crate::prelude::*;
use bevy::prelude::*;
use serde::{Deserialize, Serialize};

#[derive(Debug, Default, Component, Reflect, Clone, Deserialize, Serialize, Inspectable)]
pub struct Debug {
    #[serde(default)]
    pub message: String,
    #[serde(default)]
    pub fail: bool,
    #[serde(default)]
    pub duration: f64,
    #[serde(default)]
    pub start: f64,
}

impl BehaviorInfo for Debug {
    const TYPE: BehaviorType = BehaviorType::Action;
    const NAME: &'static str = "Debug";
    const DESC: &'static str = "Display a debug message and complete with success or failure";
}

pub fn run(
    time: Res<Time>,
    mut commands: Commands,
    mut debug_actions: Query<(Entity, &mut Debug, &mut BehaviorRunning), BehaviorRunQuery>,
) {
    for (entity, mut debug_action, mut running) in &mut debug_actions {
        if !running.on_enter_handled {
            running.on_enter_handled = true;
            debug_action.start = time.elapsed_seconds();
            debug!("[{}] RUNNING {}", entity.id(), debug_action.message);
        }
        let duration = time.elapsed_seconds() - debug_action.start;
        if duration > debug_action.duration {
            if debug_action.fail {
                commands.entity(entity).insert(BehaviorFailure);
            } else {
                commands.entity(entity).insert(BehaviorSuccess);
            }
        }
    }
}
