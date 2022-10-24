use crate::{BehaviorFailure, BehaviorInfo, BehaviorRunQuery, BehaviorSuccess, BehaviorType};
use bevy::prelude::*;

#[derive(Default, Component, Reflect, Clone)]
pub struct DebugAction {
    pub message: String,
    pub fail: bool,
    pub repeat: u8,
}

impl BehaviorInfo for DebugAction {
    const TYPE: BehaviorType = BehaviorType::Action;
    const NAME: &'static str = "Debug Action";
    const DESC: &'static str = "Display a debug message and complete with success or failure";
}

pub fn run(
    mut commands: Commands,
    mut debug_actions: Query<(Entity, &mut DebugAction), BehaviorRunQuery>,
) {
    for (entity, mut debug_action) in &mut debug_actions {
        debug!("{}: {}", debug_action.repeat, debug_action.message);
        if debug_action.repeat == 0 {
            if debug_action.fail {
                commands.entity(entity).insert(BehaviorFailure);
            } else {
                commands.entity(entity).insert(BehaviorSuccess);
            }
        } else {
            debug_action.repeat -= 1;
        }
    }
}
