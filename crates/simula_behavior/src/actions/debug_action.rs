use crate::{
    BehaviorFailure, BehaviorInfo, BehaviorRunQuery, BehaviorRunning, BehaviorSuccess, BehaviorType,
};
use bevy::prelude::*;
use serde::{Deserialize, Serialize};

#[derive(Debug, Default, Component, Reflect, Clone, Deserialize, Serialize)]
pub struct DebugAction {
    #[serde(default)]
    pub message: String,
    #[serde(default)]
    pub fail: bool,
    #[serde(default)]
    pub duration: f64,
    #[serde(default)]
    pub start: f64,
}

impl BehaviorInfo for DebugAction {
    const TYPE: BehaviorType = BehaviorType::Action;
    const NAME: &'static str = "Debug Action";
    const DESC: &'static str = "Display a debug message and complete with success or failure";
}

pub fn run(
    time: Res<Time>,
    mut commands: Commands,
    mut debug_actions: Query<(Entity, &mut DebugAction, &mut BehaviorRunning), BehaviorRunQuery>,
) {
    for (entity, mut debug_action, mut running) in &mut debug_actions {
        if !running.on_enter_handled {
            running.on_enter_handled = true;
            debug_action.start = time.seconds_since_startup();
        }

        let duration = time.seconds_since_startup() - debug_action.start;

        if duration > debug_action.duration {
            if debug_action.fail {
                commands.entity(entity).insert(BehaviorFailure);
            } else {
                commands.entity(entity).insert(BehaviorSuccess);
            }
        }

        debug!(
            "[{}] RUNNING #{} {}",
            entity.id(),
            duration,
            debug_action.message
        );
    }
}
