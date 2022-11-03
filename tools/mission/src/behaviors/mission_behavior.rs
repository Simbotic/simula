use crate::behaviors;
use bevy::{ecs::system::EntityCommands, prelude::*, reflect::TypeUuid};
use serde::{Deserialize, Serialize};
use simula_behavior::prelude::*;

pub struct MissionBehaviorPlugin;

impl Plugin for MissionBehaviorPlugin {
    fn build(&self, app: &mut App) {
        app.add_startup_system(setup)
            .add_system(behavior_loader::<MissionBehavior>)
            .add_system(behaviors::agent_rest::run)
            .add_system(behaviors::agent_work::run);
    }
}

#[derive(Debug, Serialize, Deserialize, TypeUuid)]
#[uuid = "5c3fbd4c-5359-11ed-9c5d-02a179e5df2b"]
pub enum MissionBehavior {
    Debug(Debug),
    Selector(Selector),
    Sequencer(Sequencer),
    UntilAll(UntilAll),
    UntilAny(UntilAny),
    Repeater(Repeater),
    Inverter(Inverter),
    Succeeder(Succeeder),
    Delay(Delay),
    AgentRest(behaviors::agent_rest::AgentRest),
    AgentWork(behaviors::agent_work::AgentWork),
}

impl Default for MissionBehavior {
    fn default() -> Self {
        Self::Debug(Debug::default())
    }
}

impl BehaviorSpawner for MissionBehavior {
    fn insert(&self, commands: &mut EntityCommands) {
        match self {
            MissionBehavior::Debug(data) => BehaviorInfo::insert_with(commands, data),
            MissionBehavior::Selector(data) => BehaviorInfo::insert_with(commands, data),
            MissionBehavior::Sequencer(data) => BehaviorInfo::insert_with(commands, data),
            MissionBehavior::UntilAll(data) => BehaviorInfo::insert_with(commands, data),
            MissionBehavior::UntilAny(data) => BehaviorInfo::insert_with(commands, data),
            MissionBehavior::Repeater(data) => BehaviorInfo::insert_with(commands, data),
            MissionBehavior::Inverter(data) => BehaviorInfo::insert_with(commands, data),
            MissionBehavior::Succeeder(data) => BehaviorInfo::insert_with(commands, data),
            MissionBehavior::Delay(data) => BehaviorInfo::insert_with(commands, data),
            MissionBehavior::AgentRest(data) => BehaviorInfo::insert_with(commands, data),
            MissionBehavior::AgentWork(data) => BehaviorInfo::insert_with(commands, data),
        }
    }
}

fn setup() {}

pub fn create_from_data(parent: Option<Entity>, commands: &mut Commands) -> BehaviorTree {
    let document = BehaviorDocument {
        root: BTNode(
            "Do a few times".to_string(),
            MissionBehavior::Repeater(Repeater {
                repeat: Repeat::Times(2),
                ..default()
            }),
            vec![BTNode(
                "In this order".to_string(),
                MissionBehavior::Sequencer(Sequencer::default()),
                vec![
                    BTNode(
                        "An action".to_string(),
                        MissionBehavior::Debug(Debug {
                            message: "Hello, from DebugMessage0!".to_string(),
                            ..default()
                        }),
                        vec![],
                    ),
                    BTNode(
                        "Another action".to_string(),
                        MissionBehavior::Debug(Debug {
                            message: "Hello, from DebugMessage1!".to_string(),
                            ..default()
                        }),
                        vec![],
                    ),
                ],
            )],
        ),
    };
    BehaviorTree::from_document(parent, commands, &document)
}
