use bevy::prelude::*;
use serde::{Deserialize, Serialize};
use simula_behavior::prelude::*;

use crate::{common::Robot, components::robber::*};

#[derive(Debug, Default, Component, Reflect, Clone, Deserialize, Serialize)]
pub struct RobberRestAction;

impl BehaviorInfo for RobberRestAction {
    const TYPE: BehaviorType = BehaviorType::Action;
    const NAME: &'static str = "RobberRestAction";
    const DESC: &'static str = "Make the Robber rest if the Robber has no energy";
}

pub const ROBBER_REST_SPEED: u64 = (ROBBER_STARTING_ENERGY / 20) as u64;

pub fn run(
    mut commands: Commands,
    action_query: Query<(Entity, &RobberRestAction, &BehaviorNode), BehaviorRunQuery>,
    mut query: Query<&mut Robber, With<RobberRest>>,
) {
    for (action_entity, _, node) in &action_query {
        if let Some(robber_entity) = node.tree {
            if let Ok(mut robber) = query.get_mut(robber_entity) {
                let robber_energy = robber.get_energy();
                if robber_energy < ROBBER_STARTING_ENERGY {
                    robber.set_energy(robber_energy + ROBBER_REST_SPEED);
                } else {
                    commands.entity(robber_entity).remove::<RobberRest>();
                    commands.entity(robber_entity).insert(RobberRun);
                    info!("[Robber {:?}] Started to Run", robber_entity);
                    return;
                }
            }
        }
        commands.entity(action_entity).insert(BehaviorSuccess);
    }
}
