use bevy::prelude::*;
use serde::{Deserialize, Serialize};
use simula_behavior::prelude::*;

use crate::{
    behavior_trees::bank::bank_stealed::BankStealed,
    behaviors::rest::RobotRest,
    common::Robot,
    components::{bank::*, robber::*},
};

#[derive(Debug, Default, Component, Reflect, Clone, Deserialize, Serialize)]
pub struct RobberStealAction;

impl BehaviorInfo for RobberStealAction {
    const TYPE: BehaviorType = BehaviorType::Action;
    const NAME: &'static str = "RobberStealAction";
    const DESC: &'static str = "The Bank gets stealed by the Robber";
}

pub const ROBBER_STEAL_RADIUS: f32 = 1.0;
pub const STEAL_ENERGY_COST: u64 = (ROBBER_STARTING_ENERGY / 4) as u64;

pub fn run(
    mut commands: Commands,
    action_query: Query<(Entity, &RobberStealAction, &BehaviorNode), BehaviorRunQuery>,
    mut query: Query<(&Transform, &mut Robber)>,
    mut bank_query: Query<(Entity, &Transform, &mut Bank), Without<BankStealed>>,
) {
    for (action_entity, _, node) in &action_query {
        if let Some(robber_entity) = node.tree {
            if let Ok((robber_transform, mut robber)) = query.get_mut(robber_entity) {
                for (bank_entity, bank_transform, mut bank) in bank_query.iter_mut() {
                    let robber_money = robber.get_money();
                    let robber_energy = robber.get_energy();
                    let distance =
                        (bank_transform.translation - robber_transform.translation).length();
                    if robber_energy >= STEAL_ENERGY_COST && distance <= ROBBER_STEAL_RADIUS {
                        let bank_money = bank.get_money();
                        let steal_amount = if ROBBER_STARTING_MONEY <= bank_money {
                            ROBBER_STARTING_MONEY
                        } else {
                            bank_money
                        };
                        robber.set_money(robber_money + steal_amount);
                        robber.set_energy(robber_energy - STEAL_ENERGY_COST);
                        bank.set_money(bank_money - steal_amount);
                        commands
                            .entity(bank_entity)
                            .remove::<RobotRest>()
                            .insert(BankStealed);
                        info!(
                            "[Robber {:?}] Stealed Bank with {} money using {} energy",
                            robber_entity, steal_amount, STEAL_ENERGY_COST
                        );
                        info!(
                            "[Bank {:?}] Lost {} money for being stealed by Robber",
                            bank_entity, steal_amount
                        );
                        commands.entity(action_entity).insert(BehaviorSuccess);
                    }
                }
            }
        }
    }
}
