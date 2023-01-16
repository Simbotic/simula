use bevy::prelude::*;
use serde::{Deserialize, Serialize};
use simula_behavior::prelude::*;

use crate::{
    common::Robot,
    cop::{CopChase, CopRest, COP_STARTING_ENERGY},
    CanRotate,
};

use super::Cop;

#[derive(Debug, Default, Component, Reflect, Clone, Deserialize, Serialize, Inspectable)]
pub struct CopRestAction;

impl BehaviorInfo for CopRestAction {
    const TYPE: BehaviorType = BehaviorType::Action;
    const NAME: &'static str = "CopRestAction";
    const DESC: &'static str = "Make the Cop rest if the Cop has no energy";
}

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
                    cop.set_energy(cop_energy + 2);
                } else {
                    commands.entity(cop_entity).remove::<CopRest>();
                    commands.entity(cop_entity).insert(CopChase);
                    commands.entity(cop_entity).insert(CanRotate);
                    commands.entity(action_entity).insert(BehaviorFailure);
                    info!("[Cop {:?}] Started to Chase", cop_entity);
                    return;
                }
            }
        }
        commands.entity(action_entity).insert(BehaviorSuccess);
    }
}
