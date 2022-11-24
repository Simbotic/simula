use super::{
    agent_purchase, agent_rest, agent_work, dead, machine_production,
    quest_behavior::{QuestBehavior, QuestBehaviorPlugin},
};
use bevy::{ecs::system::EntityCommands, prelude::*, reflect::TypeUuid};
use serde::{Deserialize, Serialize};
use simula_behavior::prelude::*;

use crate::MissionToken;

pub struct MissionBehaviorPlugin;

impl Plugin for MissionBehaviorPlugin {
    fn build(&self, app: &mut App) {
        app.add_startup_system(setup)
            .add_system(behavior_loader::<MissionBehavior>)
            .add_system(agent_rest::run)
            .add_system(dead::run)
            .add_system(subtree::run::<QuestBehavior>)
            .add_plugin(QuestBehaviorPlugin)
            .add_plugin(agent_work::AgentWorkNodePlugin(MissionToken::default()))
            .add_plugin(agent_purchase::AgentPurchaseNodePlugin(
                MissionToken::default(),
            ))
            .add_plugin(machine_production::MachineProductionNodePlugin(
                MissionToken::default(),
            ));
    }
}

#[derive(Debug, Serialize, Deserialize, TypeUuid, Clone)]
#[uuid = "5c3fbd4c-5359-11ed-9c5d-02a179e5df2b"]
pub enum MissionBehavior {
    Debug(Debug),
    Selector(Selector),
    Sequencer(Sequencer),
    All(All),
    Any(Any),
    Repeater(Repeater),
    Inverter(Inverter),
    Succeeder(Succeeder),
    Delay(Delay),
    Quest(Subtree<QuestBehavior>),
    AgentRest(agent_rest::AgentRest),
    AgentDead(dead::Dead),
    AgentWork(agent_work::AgentWork),
    AgentPurchase(agent_purchase::AgentPurchase),
    MachineProduction(machine_production::MachineProduction),
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
            MissionBehavior::All(data) => BehaviorInfo::insert_with(commands, data),
            MissionBehavior::Any(data) => BehaviorInfo::insert_with(commands, data),
            MissionBehavior::Repeater(data) => BehaviorInfo::insert_with(commands, data),
            MissionBehavior::Inverter(data) => BehaviorInfo::insert_with(commands, data),
            MissionBehavior::Succeeder(data) => BehaviorInfo::insert_with(commands, data),
            MissionBehavior::Delay(data) => BehaviorInfo::insert_with(commands, data),
            MissionBehavior::Quest(data) => BehaviorInfo::insert_with(commands, data),
            MissionBehavior::AgentRest(data) => BehaviorInfo::insert_with(commands, data),
            MissionBehavior::AgentDead(data) => BehaviorInfo::insert_with(commands, data),
            MissionBehavior::AgentWork(data) => BehaviorInfo::insert_with(commands, data),
            MissionBehavior::AgentPurchase(data) => BehaviorInfo::insert_with(commands, data),
            MissionBehavior::MachineProduction(data) => BehaviorInfo::insert_with(commands, data),
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
