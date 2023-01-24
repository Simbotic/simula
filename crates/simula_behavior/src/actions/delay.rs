use crate::prelude::*;
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
    mut delays: Query<(Entity, &mut Delay, &mut BehaviorRunning), BehaviorRunQuery>,
) {
    for (entity, mut delay, mut running) in &mut delays {
        if !running.on_enter_handled {
            running.on_enter_handled = true;
            delay.start = time.elapsed_seconds_f64();
        }
        let duration = time.elapsed_seconds_f64() - delay.start;
        if duration > delay.duration {
            commands.entity(entity).insert(BehaviorSuccess);
        }
    }
}
