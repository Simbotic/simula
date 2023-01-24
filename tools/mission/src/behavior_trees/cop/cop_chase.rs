use bevy::prelude::*;
use serde::{Deserialize, Serialize};
use simula_behavior::prelude::*;

use crate::{
    common::Robot,
    cop::{CopChase, CopRest},
    CanRotate,
};

use super::Cop;

#[derive(Debug, Default, Component, Reflect, Clone, Deserialize, Serialize)]
pub struct CopChaseAction;

impl BehaviorInfo for CopChaseAction {
    const TYPE: BehaviorType = BehaviorType::Action;
    const NAME: &'static str = "CopChaseAction";
    const DESC: &'static str = "Move the Cop if the Cop has energy";
}

pub fn run(
    mut commands: Commands,
    action_query: Query<(Entity, &CopChaseAction, &BehaviorNode), BehaviorRunQuery>,
    mut query: Query<&mut Cop, With<CopChase>>,
) {
    for (action_entity, _, node) in &action_query {
        if let Some(cop_entity) = node.tree {
            if let Ok(mut cop) = query.get_mut(cop_entity) {
                let cop_energy = cop.get_energy();
                if cop_energy > 0 {
                    cop.set_energy(cop_energy - 1);
                } else {
                    commands
                        .entity(cop_entity)
                        .remove::<CanRotate>()
                        .remove::<CopChase>()
                        .insert(CopRest);
                    commands.entity(action_entity).insert(BehaviorFailure);
                    info!("[Cop {:?}] Started to Rest", cop_entity);
                    return;
                }
            }
        }
        commands.entity(action_entity).insert(BehaviorSuccess);
    }
}
