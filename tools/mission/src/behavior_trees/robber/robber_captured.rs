use bevy::prelude::*;
use serde::{Deserialize, Serialize};
use simula_behavior::prelude::*;

use crate::{behaviors::movement::RobotMove, components::robber::*};

#[derive(Debug, Default, Component, Reflect, Clone, Deserialize, Serialize)]
pub struct RobberCapturedAction;

impl BehaviorInfo for RobberCapturedAction {
    const TYPE: BehaviorType = BehaviorType::Action;
    const NAME: &'static str = "RobberCapturedAction";
    const DESC: &'static str = "Handle when the Robber is captured";
}

#[derive(Component)]
#[component(storage = "SparseSet")]
pub struct RobberCaptured;

pub const ROBBER_CAPTURED_TICK_DURATION: u64 = 200;

pub fn run(
    mut commands: Commands,
    action_query: Query<(Entity, &RobberCapturedAction, &BehaviorNode), BehaviorRunQuery>,
    query: Query<&mut Robber, With<RobberCaptured>>,
    mut status_duration: Local<u64>,
) {
    for (action_entity, _, node) in &action_query {
        if let Some(robber_entity) = node.tree {
            if query.get(robber_entity).is_ok() {
                *status_duration += 1;
                if *status_duration > ROBBER_CAPTURED_TICK_DURATION {
                    *status_duration = 0;
                    commands
                        .entity(robber_entity)
                        .remove::<RobberCaptured>()
                        .insert(RobotMove);
                    info!(
                        "[Robber {:?}] Released from Capture. Started to Run",
                        robber_entity
                    );
                }
            }
        }
        commands.entity(action_entity).insert(BehaviorSuccess);
    }
}
