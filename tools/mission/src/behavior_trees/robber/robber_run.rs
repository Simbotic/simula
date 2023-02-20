use bevy::prelude::*;
use serde::{Deserialize, Serialize};
use simula_behavior::prelude::*;

use crate::{
    common::{Movement, Robot},
    components::robber::*,
    utils::calculate_movement,
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
    mut query: Query<(&mut Robber, &mut Movement, &mut Transform), With<RobberRun>>,
    time: Res<Time>,
) {
    for (action_entity, _, node) in &action_query {
        if let Some(robber_entity) = node.tree {
            if let Ok((mut robber, mut movement, mut transform)) = query.get_mut(robber_entity) {
                let robber_energy = robber.get_energy();
                if robber_energy > 0 {
                    calculate_movement(&time, &mut transform, &mut movement);
                    robber.set_energy(robber_energy - 1);
                } else {
                    commands
                        .entity(robber_entity)
                        .remove::<RobberRun>()
                        .insert(RobberRest);
                    info!("[Robber {:?}] Started to Rest", robber_entity);
                    return;
                }
            }
        }
        commands.entity(action_entity).insert(BehaviorSuccess);
    }
}
