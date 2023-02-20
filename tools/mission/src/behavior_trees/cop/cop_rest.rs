use bevy::prelude::*;
use serde::{Deserialize, Serialize};
use simula_behavior::prelude::*;

use crate::{common::Robot, components::cop::*};

#[derive(Debug, Default, Component, Reflect, Clone, Deserialize, Serialize)]
pub struct CopRestAction;

impl BehaviorInfo for CopRestAction {
    const TYPE: BehaviorType = BehaviorType::Action;
    const NAME: &'static str = "CopRestAction";
    const DESC: &'static str = "Make the Cop rest if the Cop has no energy";
}

pub const COP_REST_SPEED: u64 = (COP_STARTING_ENERGY / 20) as u64;

pub fn run(
    mut commands: Commands,
    action_query: Query<(Entity, &CopRestAction, &BehaviorNode), BehaviorRunQuery>,
    mut query: Query<&mut Cop, With<CopRest>>,
) {
    for (action_entity, _, node) in &action_query {
        if let Some(cop_entity) = node.tree {
            if let Ok(mut cop) = query.get_mut(cop_entity) {
                let cop_energy = cop.get_energy();
                if cop_energy < COP_STARTING_ENERGY {
                    cop.set_energy(cop_energy + COP_REST_SPEED);
                } else {
                    commands.entity(cop_entity).remove::<CopRest>();
                    commands.entity(cop_entity).insert(CopChase);
                    info!("[Cop {:?}] Started to Chase", cop_entity);
                    return;
                }
            }
        }
        commands.entity(action_entity).insert(BehaviorSuccess);
    }
}
