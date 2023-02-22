use bevy::{ecs::system::EntityCommands, prelude::*, reflect::TypeUuid};
use serde::{Deserialize, Serialize};
use simula_behavior::prelude::*;

use crate::{
    behaviors::{movement, rest},
    components::citizen::Citizen,
};

#[derive(Serialize, Deserialize, TypeUuid, Debug, Clone)]
#[uuid = "a5786386-aed8-4f1e-b74b-1d473a8e88c8"]
pub enum CitizenBehavior {
    Debug(Debug),
    Delay(Delay),
    Selector(Selector),
    Sequencer(Sequencer),
    Repeater(Repeater),
    Inverter(Inverter),
    Any(Any),
    All(All),
    CitizenMove(movement::RobotMoveAction),
    CitizenRest(rest::RobotRestAction),
}

impl Default for CitizenBehavior {
    fn default() -> Self {
        Self::Debug(Debug::default())
    }
}

impl BehaviorSpawner for CitizenBehavior {
    fn insert(&self, commands: &mut EntityCommands) {
        match self {
            CitizenBehavior::Debug(action) => BehaviorInfo::insert_with(commands, action),
            CitizenBehavior::Delay(action) => BehaviorInfo::insert_with(commands, action),
            CitizenBehavior::Selector(selector) => BehaviorInfo::insert_with(commands, selector),
            CitizenBehavior::Sequencer(sequence) => BehaviorInfo::insert_with(commands, sequence),
            CitizenBehavior::Repeater(repeater) => BehaviorInfo::insert_with(commands, repeater),
            CitizenBehavior::Inverter(inverter) => BehaviorInfo::insert_with(commands, inverter),
            CitizenBehavior::Any(any) => BehaviorInfo::insert_with(commands, any),
            CitizenBehavior::All(all) => BehaviorInfo::insert_with(commands, all),
            CitizenBehavior::CitizenMove(action) => BehaviorInfo::insert_with(commands, action),
            CitizenBehavior::CitizenRest(action) => BehaviorInfo::insert_with(commands, action),
        }
    }
}

pub fn setup_behavior(
    mut commands: Commands,
    query: Query<Entity, (With<Citizen>, Without<BehaviorTree>)>,
    asset_server: Res<AssetServer>,
) {
    for citizen in query.iter() {
        let document: Handle<BehaviorAsset> =
            asset_server.load("behaviors/mission/citizen.bht.ron");
        let behavior = BehaviorTree::from_asset::<CitizenBehavior>(None, &mut commands, document);
        if let Some(root) = behavior.root {
            commands.entity(root).insert(BehaviorCursor);
        }
        commands
            .entity(citizen)
            .push_children(&[behavior.root.unwrap()])
            .insert(behavior);
    }
}

pub struct CitizenBehaviorPlugin;

impl Plugin for CitizenBehaviorPlugin {
    fn build(&self, app: &mut App) {
        app.add_system(behavior_loader::<CitizenBehavior>)
            .add_system(setup_behavior)
            .add_system(movement::run::<Citizen>)
            .add_system(rest::run::<Citizen>);
    }
}
