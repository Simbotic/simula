use bevy::prelude::*;
use serde::{Deserialize, Serialize};
use simula_behavior::prelude::*;

#[derive(Debug, Default, Component, Reflect, Clone, Serialize, Deserialize, Inspectable)]
pub struct Dead {}

impl BehaviorInfo for Dead {
    const TYPE: BehaviorType = BehaviorType::Action;
    const NAME: &'static str = "Dead";
    const DESC: &'static str = "Stop execution indefinitely";
}

pub fn run(
    mut _commands: Commands,
    mut _dead: Query<
        (
            Entity,
            &mut Dead,
            &mut BehaviorRunning,
            &mut BehaviorNode,
        ),
        BehaviorRunQuery,
    >,
) {}
