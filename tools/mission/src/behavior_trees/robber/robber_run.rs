use bevy::prelude::*;
use serde::{Deserialize, Serialize};
use simula_behavior::prelude::*;

use crate::{
    common::Robot,
    robber::{Robber, RobberRest, RobberRun},
    CanRotate,
};

#[derive(Debug, Default, Component, Reflect, Clone, Deserialize, Serialize)]
pub struct RobberRunAction;

impl BehaviorInfo for RobberRunAction {
    const TYPE: BehaviorType = BehaviorType::Action;
    const NAME: &'static str = "RobberRunAction";
    const DESC: &'static str = "Move the Robber if the Robber has energy";
}

pub fn run(
    mut commands: Commands,
    action_query: Query<(Entity, &RobberRunAction, &BehaviorNode), BehaviorRunQuery>,
    mut query: Query<&mut Robber, With<RobberRun>>,
) {
    for (action_entity, _, node) in &action_query {
        if let Some(robber_entity) = node.tree {
            if let Ok(mut robber) = query.get_mut(robber_entity) {
                let robber_energy = robber.get_energy();
                if robber_energy > 0 {
                    robber.set_energy(robber_energy - 1);
                } else {
                    commands
                        .entity(robber_entity)
                        .remove::<CanRotate>()
                        .remove::<RobberRun>()
                        .insert(RobberRest);
                    commands.entity(action_entity).insert(BehaviorFailure);
                    info!("[Robber {:?}] Started to Rest", robber_entity);
                    return;
                }
            }
        }
        commands.entity(action_entity).insert(BehaviorSuccess);
    }
}
