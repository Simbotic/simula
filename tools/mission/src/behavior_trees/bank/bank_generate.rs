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

#[derive(Component)]
pub struct BankGenerate;

pub fn run(
    mut commands: Commands,
    mut action_query: Query<
        (
            Entity,
            &BankGenerateAction,
            &BehaviorNode,
            &mut BehaviorRunning,
        ),
        BehaviorRunQuery,
    >,
    mut query: Query<(&mut Bank, Option<&BankGenerate>)>,
) {
    for (action_entity, _, node, mut running) in &mut action_query {
        if let Some(bank_entity) = node.tree {
            if let Ok((mut bank, generate)) = query.get_mut(bank_entity) {
                let bank_energy = bank.get_energy();
                let bank_money = bank.get_money();
                if running.on_enter_handled && generate.is_none() {
                    commands.entity(action_entity).insert(BehaviorSuccess);
                }
                if bank_energy > 0 && bank_money < BANK_MAX_MONEY {
                    if !running.on_enter_handled {
                        running.on_enter_handled = true;
                        commands.entity(bank_entity).insert(BankGenerate);
                    }
                    bank.set_money(bank_money + 1);
                    bank.set_energy(bank_energy - 1);
                } else {
                    commands.entity(bank_entity).remove::<BankGenerate>();
                    info!("[Bank {:?}] Started to Rest", bank_entity);
                    commands.entity(action_entity).insert(BehaviorSuccess);
                    return;
                }
            }
        }
    }
}
