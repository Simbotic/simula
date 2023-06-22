use crate::prelude::*;
use bevy::prelude::*;
use bevy_inspector_egui::prelude::*;
use serde::{Deserialize, Serialize};

/// A wait will succeed after a specified amount of time.
#[derive(
    Debug, Default, Component, Reflect, FromReflect, Clone, Deserialize, Serialize, InspectorOptions,
)]
#[reflect(InspectorOptions)]
pub struct Wait {
    #[serde(default)]
    #[inspector(min = 0.0, max = f64::MAX)]
    pub duration: f64,
    #[serde(skip)]
    pub start: f64,
    #[serde(skip)]
    pub ticks: u64,
}

impl BehaviorInfo for Wait {
    const TYPE: BehaviorType = BehaviorType::Action;
    const NAME: &'static str = "⌛ Wait";
    const DESC: &'static str = "Wait for a specified amount of time";
}

pub fn run(
    time: Res<Time>,
    mut commands: Commands,
    mut waits: Query<(Entity, &mut Wait, Option<&BehaviorStarted>), BehaviorRunQuery>,
) {
    for (entity, mut wait, started) in &mut waits {
        wait.ticks += 1;
        let elapsed = time.elapsed_seconds_f64();
        if started.is_some() {
            wait.start = elapsed;
        }
        if elapsed - wait.start > wait.duration - f64::EPSILON {
            commands.entity(entity).insert(BehaviorSuccess);
        }
    }
}
