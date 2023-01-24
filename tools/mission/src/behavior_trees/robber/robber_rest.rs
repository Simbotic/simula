use bevy::prelude::*;
use serde::{Deserialize, Serialize};
use simula_behavior::prelude::*;

use crate::{
    common::Robot,
    robber::{RobberRest, RobberRun, ROBBER_STARTING_ENERGY},
    CanRotate,
};

use super::Robber;

#[derive(Debug, Default, Component, Reflect, Clone, Deserialize, Serialize)]
pub struct RobberRestAction;

impl BehaviorInfo for RobberRestAction {
    const TYPE: BehaviorType = BehaviorType::Action;
    const NAME: &'static str = "RobberRestAction";
    const DESC: &'static str = "Make the Robber rest if the Robber has no energy";
}

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
                    robber.set_energy(robber_energy + 2);
                } else {
                    commands.entity(robber_entity).remove::<RobberRest>();
                    commands.entity(robber_entity).insert(RobberRun);
                    commands.entity(robber_entity).insert(CanRotate);
                    commands.entity(action_entity).insert(BehaviorFailure);
                    info!("[Robber {:?}] Started to Run", robber_entity);
                    return;
                }
            }
        }
        commands.entity(action_entity).insert(BehaviorSuccess);
    }
}
