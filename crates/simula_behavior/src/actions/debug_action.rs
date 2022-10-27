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
    pub repeat: u64,
    #[serde(default)]
    pub count: u64,
}

impl BehaviorInfo for DebugAction {
    const TYPE: BehaviorType = BehaviorType::Action;
    const NAME: &'static str = "Debug Action";
    const DESC: &'static str = "Display a debug message and complete with success or failure";
}

pub fn run(
    mut commands: Commands,
    mut debug_actions: Query<(Entity, &mut DebugAction, &mut BehaviorRunning), BehaviorRunQuery>,
) {
    for (entity, mut debug_action, mut running) in &mut debug_actions {
        if !running.on_enter_handled {
            running.on_enter_handled = true;
            debug_action.count = 0;
        }

        if debug_action.repeat > 0 && debug_action.count < debug_action.repeat {
            debug_action.count += 1;
        } else {
            if debug_action.fail {
                commands.entity(entity).insert(BehaviorFailure);
            } else {
                commands.entity(entity).insert(BehaviorSuccess);
            }
        }

        debug!(
            "[{}] RUNNING #{} {}",
            entity.id(),
            debug_action.count,
            debug_action.message
        );
    }
}
