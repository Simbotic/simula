use bevy::prelude::*;
use serde::{Deserialize, Serialize};
use simula_behavior::prelude::*;

use crate::{common::Robot, components::bank::*};

#[derive(Debug, Default, Component, Reflect, Clone, Deserialize, Serialize)]
pub struct BankGenerateAction;

impl BehaviorInfo for BankGenerateAction {
    const TYPE: BehaviorType = BehaviorType::Action;
    const NAME: &'static str = "BankGenerateAction";
    const DESC: &'static str = "Make the Bank generate money if the Bank has energy until a certain amount of money is reached.";
}

pub fn run(
    mut commands: Commands,
    action_query: Query<(Entity, &BankGenerateAction, &BehaviorNode), BehaviorRunQuery>,
    mut query: Query<&mut Bank, With<BankGenerate>>,
) {
    for (action_entity, _, node) in &action_query {
        if let Some(bank_entity) = node.tree {
            if let Ok(mut bank) = query.get_mut(bank_entity) {
                let bank_energy = bank.get_energy();
                let bank_money = bank.get_money();
                if bank_energy > 0 && bank_money < BANK_MAX_MONEY {
                    bank.set_money(bank_money + 1);
                    bank.set_energy(bank_energy - 1);
                } else {
                    commands.entity(bank_entity).remove::<BankGenerate>();
                    info!("[Bank {:?}] Started to Rest", bank_entity);
                    return;
                }
            }
        }
        commands.entity(action_entity).insert(BehaviorSuccess);
    }
}
