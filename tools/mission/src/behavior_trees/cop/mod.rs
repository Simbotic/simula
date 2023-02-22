use bevy::{ecs::system::EntityCommands, prelude::*, reflect::TypeUuid};
use serde::{Deserialize, Serialize};
use simula_behavior::prelude::*;

use crate::{
    behaviors::{movement, rest},
    components::cop::Cop,
};

pub mod cop_alerted;
pub mod cop_bribed;
pub mod cop_capture;

#[derive(Serialize, Deserialize, TypeUuid, Debug, Clone)]
#[uuid = "731ec6a7-b356-48d4-abb2-eeadee3287fd"]
pub enum CopBehavior {
    Debug(Debug),
    Delay(Delay),
    Selector(Selector),
    Sequencer(Sequencer),
    Repeater(Repeater),
    Inverter(Inverter),
    Any(Any),
    All(All),
    CopChase(movement::RobotMoveAction),
    CopCapture(cop_capture::CopCaptureAction),
    CopBribed(cop_bribed::CopBribedAction),
    CopRest(rest::RobotRestAction),
    CopAlerted(cop_alerted::CopAlertedAction),
}

impl Default for CopBehavior {
    fn default() -> Self {
        Self::Debug(Debug::default())
    }
}

impl BehaviorSpawner for CopBehavior {
    fn insert(&self, commands: &mut EntityCommands) {
        match self {
            CopBehavior::Debug(action) => BehaviorInfo::insert_with(commands, action),
            CopBehavior::Delay(action) => BehaviorInfo::insert_with(commands, action),
            CopBehavior::Selector(selector) => BehaviorInfo::insert_with(commands, selector),
            CopBehavior::Sequencer(sequence) => BehaviorInfo::insert_with(commands, sequence),
            CopBehavior::Repeater(repeater) => BehaviorInfo::insert_with(commands, repeater),
            CopBehavior::Inverter(inverter) => BehaviorInfo::insert_with(commands, inverter),
            CopBehavior::Any(any) => BehaviorInfo::insert_with(commands, any),
            CopBehavior::All(all) => BehaviorInfo::insert_with(commands, all),
            CopBehavior::CopChase(action) => BehaviorInfo::insert_with(commands, action),
            CopBehavior::CopCapture(action) => BehaviorInfo::insert_with(commands, action),
            CopBehavior::CopBribed(action) => BehaviorInfo::insert_with(commands, action),
            CopBehavior::CopRest(action) => BehaviorInfo::insert_with(commands, action),
            CopBehavior::CopAlerted(action) => BehaviorInfo::insert_with(commands, action),
        }
    }
}

pub fn setup_behavior(
    mut commands: Commands,
    query: Query<Entity, (With<Cop>, Without<BehaviorTree>)>,
    asset_server: Res<AssetServer>,
) {
    for cop in query.iter() {
        let document: Handle<BehaviorAsset> = asset_server.load("behaviors/mission/cop.bht.ron");
        let behavior = BehaviorTree::from_asset::<CopBehavior>(None, &mut commands, document);
        if let Some(root) = behavior.root {
            commands.entity(root).insert(BehaviorCursor);
        }
        commands
            .entity(cop)
            .push_children(&[behavior.root.unwrap()])
            .insert(behavior);
    }
}

pub struct CopBehaviorPlugin;

impl Plugin for CopBehaviorPlugin {
    fn build(&self, app: &mut App) {
        app.add_system(behavior_loader::<CopBehavior>)
            .add_system(setup_behavior)
            .add_system(movement::run::<Cop>)
            .add_system(cop_capture::run)
            .add_system(cop_bribed::run)
            .add_system(rest::run::<Cop>)
            .add_system(cop_alerted::run);
    }
}
