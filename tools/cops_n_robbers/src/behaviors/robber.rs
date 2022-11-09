use crate::behaviors;
use bevy::{ecs::system::EntityCommands, prelude::*, reflect::TypeUuid};
use serde::{Deserialize, Serialize};
use simula_behavior::prelude::*;

pub struct RobberBehaviorPlugin;

impl Plugin for RobberBehaviorPlugin {
    fn build(&self, app: &mut App) {
        app.add_startup_system(setup)
            .add_system(behavior_loader::<RobberBehavior>);
    }
}

#[derive(Debug, Serialize, Deserialize, TypeUuid, Clone, Inspectable)]
#[uuid = "8de2b8f4-5c96-11ed-bdf5-02a179e5df2b"]
pub enum RobberBehavior {
    Debug(Debug),
    Selector(Selector),
    Sequencer(Sequencer),
    All(All),
    Any(Any),
    Repeater(Repeater),
    Inverter(Inverter),
    Succeeder(Succeeder),
    Delay(Delay),
}

impl Default for RobberBehavior {
    fn default() -> Self {
        Self::Debug(Debug::default())
    }
}

impl BehaviorSpawner for RobberBehavior {
    fn insert(&self, commands: &mut EntityCommands) {
        match self {
            RobberBehavior::Debug(data) => BehaviorInfo::insert_with(commands, data),
            RobberBehavior::Selector(data) => BehaviorInfo::insert_with(commands, data),
            RobberBehavior::Sequencer(data) => BehaviorInfo::insert_with(commands, data),
            RobberBehavior::All(data) => BehaviorInfo::insert_with(commands, data),
            RobberBehavior::Any(data) => BehaviorInfo::insert_with(commands, data),
            RobberBehavior::Repeater(data) => BehaviorInfo::insert_with(commands, data),
            RobberBehavior::Inverter(data) => BehaviorInfo::insert_with(commands, data),
            RobberBehavior::Succeeder(data) => BehaviorInfo::insert_with(commands, data),
            RobberBehavior::Delay(data) => BehaviorInfo::insert_with(commands, data),
        }
    }
}

fn setup() {}
