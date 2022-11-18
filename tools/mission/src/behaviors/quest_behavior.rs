use crate::behaviors;
use bevy::{ecs::system::EntityCommands, prelude::*, reflect::TypeUuid};
use serde::{Deserialize, Serialize};
use simula_behavior::prelude::*;

pub struct QuestBehaviorPlugin;

impl Plugin for QuestBehaviorPlugin {
    fn build(&self, app: &mut App) {
        app.add_startup_system(setup)
            .add_system(behavior_loader::<QuestBehavior>)
            .add_system(behaviors::agent_rest::run);
            // .add_system(behaviors::agent_work::run);
    }
}

#[derive(Debug, Serialize, Deserialize, TypeUuid, Clone, Inspectable)]
#[uuid = "8de2b8f4-5c96-11ed-bdf5-02a179e5df2b"]
pub enum QuestBehavior {
    Debug(Debug),
    Selector(Selector),
    Sequencer(Sequencer),
    All(All),
    Any(Any),
    Repeater(Repeater),
    Inverter(Inverter),
    Succeeder(Succeeder),
    Delay(Delay),
    AgentRest(behaviors::agent_rest::AgentRest),
    AgentWork(behaviors::agent_work::AgentWork),
}

impl Default for QuestBehavior {
    fn default() -> Self {
        Self::Debug(Debug::default())
    }
}

impl BehaviorSpawner for QuestBehavior {
    fn insert(&self, commands: &mut EntityCommands) {
        match self {
            QuestBehavior::Debug(data) => BehaviorInfo::insert_with(commands, data),
            QuestBehavior::Selector(data) => BehaviorInfo::insert_with(commands, data),
            QuestBehavior::Sequencer(data) => BehaviorInfo::insert_with(commands, data),
            QuestBehavior::All(data) => BehaviorInfo::insert_with(commands, data),
            QuestBehavior::Any(data) => BehaviorInfo::insert_with(commands, data),
            QuestBehavior::Repeater(data) => BehaviorInfo::insert_with(commands, data),
            QuestBehavior::Inverter(data) => BehaviorInfo::insert_with(commands, data),
            QuestBehavior::Succeeder(data) => BehaviorInfo::insert_with(commands, data),
            QuestBehavior::Delay(data) => BehaviorInfo::insert_with(commands, data),
            QuestBehavior::AgentRest(data) => BehaviorInfo::insert_with(commands, data),
            QuestBehavior::AgentWork(data) => BehaviorInfo::insert_with(commands, data),
        }
    }
}

fn setup() {}
