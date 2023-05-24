use crate::prelude::*;
use bevy::prelude::*;
use serde::{Deserialize, Serialize};

/// A wait will succeed after a specified amount of time.
#[derive(Debug, Default, Component, Reflect, Clone, Deserialize, Serialize)]
pub struct Wait {
    #[serde(default)]
    pub duration: f64,
    #[serde(default)]
    pub start: f64,
}

impl BehaviorInfo for Wait {
    const TYPE: BehaviorType = BehaviorType::Action;
    const NAME: &'static str = "Wait";
    const DESC: &'static str = "Wait for a specified amount of time";
}

pub fn run(
    time: Res<Time>,
    mut commands: Commands,
    mut waits: Query<(Entity, &mut Wait, &mut BehaviorRunning), BehaviorRunQuery>,
) {
    for (entity, mut wait, mut running) in &mut waits {
        if !running.on_enter_handled {
            running.on_enter_handled = true;
            wait.start = time.elapsed_seconds_f64();
        }
        let duration = time.elapsed_seconds_f64() - wait.start;
        if duration > wait.duration {
            commands.entity(entity).insert(BehaviorSuccess);
        }
    }
}