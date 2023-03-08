use bevy::{ecs::system::EntityCommands, prelude::*, reflect::TypeUuid};
use serde::{Deserialize, Serialize};
use simula_behavior::prelude::*;

use crate::{behaviors::rest, components::bank::Bank};

pub mod bank_generate;
pub mod bank_stealed;

#[derive(Serialize, Deserialize, TypeUuid, Debug, Clone)]
#[uuid = "929645ce-09c9-489d-9c2f-c45c5d6105d5"]
pub enum BankBehavior {
    Debug(Debug),
    Delay(Delay),
    Selector(Selector),
    Sequencer(Sequencer),
    Repeater(Repeater),
    Inverter(Inverter),
    Any(Any),
    All(All),
    BankGenerate(bank_generate::BankGenerateAction),
    BankRest(rest::RobotRestAction),
    BankStealed(bank_stealed::BankStealedAction),
}

impl Default for BankBehavior {
    fn default() -> Self {
        Self::Debug(Debug::default())
    }
}

impl BehaviorSpawner for BankBehavior {
    fn insert(&self, commands: &mut EntityCommands) {
        match self {
            BankBehavior::Debug(action) => BehaviorInfo::insert_with(commands, action),
            BankBehavior::Delay(action) => BehaviorInfo::insert_with(commands, action),
            BankBehavior::Selector(selector) => BehaviorInfo::insert_with(commands, selector),
            BankBehavior::Sequencer(sequence) => BehaviorInfo::insert_with(commands, sequence),
            BankBehavior::Repeater(repeater) => BehaviorInfo::insert_with(commands, repeater),
            BankBehavior::Inverter(inverter) => BehaviorInfo::insert_with(commands, inverter),
            BankBehavior::Any(any) => BehaviorInfo::insert_with(commands, any),
            BankBehavior::All(all) => BehaviorInfo::insert_with(commands, all),
            BankBehavior::BankGenerate(action) => BehaviorInfo::insert_with(commands, action),
            BankBehavior::BankRest(action) => BehaviorInfo::insert_with(commands, action),
            BankBehavior::BankStealed(action) => BehaviorInfo::insert_with(commands, action),
        }
    }
}

pub fn setup_behavior(
    mut commands: Commands,
    query: Query<Entity, (With<Bank>, Without<BehaviorTree>)>,
    asset_server: Res<AssetServer>,
) {
    for bank in query.iter() {
        let document: Handle<BehaviorAsset> = asset_server.load("behaviors/mission/bank.bht.ron");
        let behavior = BehaviorTree::from_asset::<BankBehavior>(None, &mut commands, document);
        if let Some(root) = behavior.root {
            commands.entity(root).insert(BehaviorCursor);
        }
        commands
            .entity(bank)
            .push_children(&[behavior.root.unwrap()])
            .insert(behavior);
    }
}

pub struct BankBehaviorPlugin;

impl Plugin for BankBehaviorPlugin {
    fn build(&self, app: &mut App) {
        app.add_system(behavior_loader::<BankBehavior>)
            .add_system(setup_behavior)
            .add_system(bank_generate::run)
            .add_system(rest::run::<Bank>)
            .add_system(bank_stealed::run);
    }
}