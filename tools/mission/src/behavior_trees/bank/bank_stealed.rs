use bevy::prelude::*;
use serde::{Deserialize, Serialize};
use simula_behavior::prelude::*;

use crate::{
    behavior_trees::cop::cop_alerted::CopAlerted,
    common::Robot,
    components::{bank::*, cop::Cop},
};

#[derive(Debug, Default, Component, Reflect, Clone, Deserialize, Serialize)]
pub struct BankStealedAction;

impl BehaviorInfo for BankStealedAction {
    const TYPE: BehaviorType = BehaviorType::Action;
    const NAME: &'static str = "BankStealedAction";
    const DESC: &'static str = "Handle the state of the Bank";
}

#[derive(Component)]
#[component(storage = "SparseSet")]
pub struct BankStealed;

pub const BANK_ALERT_ENERGY_COST: f32 = BANK_STARTING_ENERGY / 2.0;

pub fn run(
    mut commands: Commands,
    action_query: Query<(Entity, &BankStealedAction, &BehaviorNode), BehaviorRunQuery>,
    mut query: Query<&mut Bank, With<BankStealed>>,
    cop_query: Query<Entity, (With<Cop>, Without<CopAlerted>)>,
) {
    for (action_entity, _, node) in &action_query {
        if let Some(bank_entity) = node.tree {
            if let Ok(mut bank) = query.get_mut(bank_entity) {
                let bank_energy = bank.get_energy();
                if bank_energy >= BANK_ALERT_ENERGY_COST {
                    bank.set_energy(bank_energy - BANK_ALERT_ENERGY_COST);
                    cop_query.iter().for_each(|cop_entity| {
                        commands.entity(cop_entity).insert(CopAlerted);
                    });
                }
                commands.entity(bank_entity).remove::<BankStealed>();
                info!(
                    "[Bank {:?}] Alerted the Cops after being Stealed",
                    bank_entity
                );
                commands.entity(action_entity).insert(BehaviorSuccess);
            } else {
                commands.entity(action_entity).insert(BehaviorFailure);
            }
        }
    }
}
