use crate::{BehaviorInfo, BehaviorRunQuery, BehaviorRunning, BehaviorSuccess, BehaviorType};
use bevy::prelude::*;
use serde::{Deserialize, Serialize};

/// A delay will succeed after a specified amount of time.
#[derive(Debug, Default, Component, Reflect, Clone, Deserialize, Serialize)]
pub struct Delay {
    #[serde(default)]
    pub duration: f64,
    #[serde(default)]
    pub start: f64,
}

impl BehaviorInfo for Delay {
    const TYPE: BehaviorType = BehaviorType::Action;
    const NAME: &'static str = "Delay";
    const DESC: &'static str = "Delay for a specified amount of time";
}

pub fn run(
    time: Res<Time>,
    mut commands: Commands,
    mut debug_actions: Query<(Entity, &mut Delay, &mut BehaviorRunning), BehaviorRunQuery>,
) {
    for (entity, mut debug_action, mut running) in &mut debug_actions {
        if !running.on_enter_handled {
            running.on_enter_handled = true;
            debug_action.start = time.seconds_since_startup();
        }
        let duration = time.seconds_since_startup() - debug_action.start;
        if duration > debug_action.duration {
            commands.entity(entity).insert(BehaviorSuccess);
        }
        debug!("[{}] RUNNING #{}", entity.id(), duration,);
    }
}
