use bevy::prelude::*;
use serde::{Deserialize, Serialize};
use simula_behavior::prelude::*;

use crate::{
    common::{CanRotate, Robot},
    cop::{Cop, CopBribed, CopChase, CopRest},
    robber::{Robber, RobberRun},
};

#[derive(Debug, Default, Component, Reflect, Clone, Deserialize, Serialize)]
pub struct RobberBribeAction;

impl BehaviorInfo for RobberBribeAction {
    const TYPE: BehaviorType = BehaviorType::Action;
    const NAME: &'static str = "RobberBribeAction";
    const DESC: &'static str = "The Cop gets bribed by the Robber, loses energy and it's unable to capture the Robber for a while";
}

pub const BRIBE_COST: u64 = 100;
pub const BRIBE_ENERGY_COST: u64 = 50;
pub const ROBBER_BRIBE_RANGE: f32 = 0.35;

pub fn run(
    mut commands: Commands,
    action_query: Query<(Entity, &RobberBribeAction, &BehaviorNode), BehaviorRunQuery>,
    mut query: Query<(&Transform, &mut Robber), With<RobberRun>>,
    mut cop_query: Query<(Entity, &Transform, &mut Cop), Without<CopBribed>>,
) {
    for (action_entity, _, node) in &action_query {
        if let Some(robber_entity) = node.tree {
            if let Ok((robber_transform, mut robber)) = query.get_mut(robber_entity) {
                for (cop_entity, cop_transform, mut cop) in cop_query.iter_mut() {
                    let robber_money = robber.get_money();
                    let robber_energy = robber.get_energy();
                    if robber_money < BRIBE_COST || robber_energy < BRIBE_ENERGY_COST {
                        commands.entity(action_entity).insert(BehaviorFailure);
                        return;
                    }
                    let cop_rotation = robber_transform.rotation;
                    let robber_rotation = cop_transform.rotation;
                    let rotation_diff = cop_rotation.y - robber_rotation.y;
                    if rotation_diff < ROBBER_BRIBE_RANGE && rotation_diff > -ROBBER_BRIBE_RANGE {
                        let cop_money = cop.get_money();
                        let cop_energy = cop.get_energy();
                        let bribe_cop_penalty = BRIBE_ENERGY_COST * 2;
                        robber.set_money(robber_money - BRIBE_COST);
                        robber.set_energy(robber_energy - BRIBE_ENERGY_COST);
                        cop.set_money(cop_money + BRIBE_COST);
                        if cop_energy > bribe_cop_penalty {
                            cop.set_energy(cop_energy - bribe_cop_penalty);
                        } else {
                            cop.set_energy(0);
                        }
                        commands
                            .entity(cop_entity)
                            .remove::<CanRotate>()
                            .remove::<CopChase>()
                            .remove::<CopRest>()
                            .insert(CopBribed);
                        info!(
                            "[Robber {:?}] Bribed Cop with {} money using {} energy",
                            robber_entity, BRIBE_COST, BRIBE_ENERGY_COST
                        );
                        info!(
                            "[Cop {:?}] Got bribed with {} money. Lost {} energy admiring the money",
                            cop_entity, BRIBE_COST, bribe_cop_penalty
                        );
                    }
                }
            }
        }
        commands.entity(action_entity).insert(BehaviorSuccess);
    }
}
