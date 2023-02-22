use bevy::{ecs::system::EntityCommands, prelude::*, reflect::TypeUuid};
use serde::{Deserialize, Serialize};
use simula_behavior::prelude::*;

use crate::{
    behaviors::{movement, rest},
    components::robber::Robber,
};

pub mod robber_bribe;
pub mod robber_captured;
pub mod robber_steal;

#[derive(Serialize, Deserialize, TypeUuid, Debug, Clone)]
#[uuid = "99bd66e1-5e2e-40fb-a639-5fd56667b752"]
pub enum RobberBehavior {
    Debug(Debug),
    Delay(Delay),
    Selector(Selector),
    Sequencer(Sequencer),
    Repeater(Repeater),
    Inverter(Inverter),
    Any(Any),
    All(All),
    RobberRun(movement::RobotMoveAction),
    RobberBribe(robber_bribe::RobberBribeAction),
    RobberCaptured(robber_captured::RobberCapturedAction),
    RobberRest(rest::RobotRestAction),
    RobberSteal(robber_steal::RobberStealAction),
}

impl Default for RobberBehavior {
    fn default() -> Self {
        Self::Debug(Debug::default())
    }
}

impl BehaviorSpawner for RobberBehavior {
    fn insert(&self, commands: &mut EntityCommands) {
        match self {
            RobberBehavior::Debug(action) => BehaviorInfo::insert_with(commands, action),
            RobberBehavior::Delay(action) => BehaviorInfo::insert_with(commands, action),
            RobberBehavior::Selector(selector) => BehaviorInfo::insert_with(commands, selector),
            RobberBehavior::Sequencer(sequence) => BehaviorInfo::insert_with(commands, sequence),
            RobberBehavior::Repeater(repeater) => BehaviorInfo::insert_with(commands, repeater),
            RobberBehavior::Inverter(inverter) => BehaviorInfo::insert_with(commands, inverter),
            RobberBehavior::Any(any) => BehaviorInfo::insert_with(commands, any),
            RobberBehavior::All(all) => BehaviorInfo::insert_with(commands, all),
            RobberBehavior::RobberRun(action) => BehaviorInfo::insert_with(commands, action),
            RobberBehavior::RobberBribe(action) => BehaviorInfo::insert_with(commands, action),
            RobberBehavior::RobberCaptured(action) => BehaviorInfo::insert_with(commands, action),
            RobberBehavior::RobberRest(action) => BehaviorInfo::insert_with(commands, action),
            RobberBehavior::RobberSteal(action) => BehaviorInfo::insert_with(commands, action),
        }
    }
}

pub fn setup_behavior(
    mut commands: Commands,
    query: Query<Entity, (With<Robber>, Without<BehaviorTree>)>,
    asset_server: Res<AssetServer>,
) {
    for robber in query.iter() {
        let document: Handle<BehaviorAsset> = asset_server.load("behaviors/mission/robber.bht.ron");
        let behavior = BehaviorTree::from_asset::<RobberBehavior>(None, &mut commands, document);
        if let Some(root) = behavior.root {
            commands.entity(root).insert(BehaviorCursor);
        }
        commands
            .entity(robber)
            .push_children(&[behavior.root.unwrap()])
            .insert(behavior);
    }
}

pub struct RobberBehaviorPlugin;

impl Plugin for RobberBehaviorPlugin {
    fn build(&self, app: &mut App) {
        app.add_system(behavior_loader::<RobberBehavior>)
            .add_system(setup_behavior)
            .add_system(movement::run::<Robber>)
            .add_system(robber_bribe::run)
            .add_system(robber_captured::run)
            .add_system(rest::run::<Robber>)
            .add_system(robber_steal::run);
    }
}
