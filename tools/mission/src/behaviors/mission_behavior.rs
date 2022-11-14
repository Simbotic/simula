use super::{
    agent_rest, agent_work, machine_production,
    quest_behavior::{QuestBehavior, QuestBehaviorPlugin},
};
use bevy::{ecs::system::EntityCommands, prelude::*, reflect::TypeUuid};
use serde::{Deserialize, Serialize};
use simula_behavior::prelude::*;

use simula_mission::{asset_ui::AssetInfo, asset::Asset, asset::Amount};

pub struct MissionBehaviorPlugin;

impl Plugin for MissionBehaviorPlugin {
    fn build(&self, app: &mut App) {
        app.add_startup_system(setup)
            .add_system(behavior_loader::<MissionBehavior>)
            .add_system(agent_rest::run)
            .add_system(agent_work::run)
            .add_system(subtree::run::<QuestBehavior>)
            .add_plugin(QuestBehaviorPlugin)
            .add_plugin(machine_production::MachineProductionNodePlugin(MissionToken::default()));
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
    AgentWork(agent_work::AgentWork),
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
            MissionBehavior::AgentWork(data) => BehaviorInfo::insert_with(commands, data),
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

// remove missiontoken from here
#[derive(Debug, Inspectable, Reflect, Component, Clone, PartialEq)]
#[reflect(Component)]
pub enum MissionToken {
    Time(Asset<1000, 0>),
    Trust(Asset<1000, 1>),
    Energy(Asset<1000, 2>),
    Labor(Asset<1000, 3>),
}

impl Default for MissionToken {
    fn default() -> Self {
        Self::Time(0.into())
    }
}

impl AssetInfo for MissionToken {
    fn name(&self) -> &'static str {
        match self {
            MissionToken::Time(_) => "Time",
            MissionToken::Trust(_) => "Trust",
            MissionToken::Energy(_) => "Energy",
            MissionToken::Labor(_) => "Labor",
        }
    }

    fn icon_dir(&self) -> &'static str {
        match self {
            MissionToken::Time(_) => "../assets/mission/Balance.png",
            MissionToken::Trust(_) => "../assets/mission/Money - Cash.png",
            MissionToken::Energy(_) => "../assets/mission/Money - Coins.png",
            MissionToken::Labor(_) => "../assets/mission/labor-icon.png",
        }
    }

    fn amount(&self) -> Amount {
        match self {
            MissionToken::Time(asset) => asset.0,
            MissionToken::Trust(asset) => asset.0,
            MissionToken::Energy(asset) => asset.0,
            MissionToken::Labor(asset) => asset.0,
        }
    }

    fn is_draggable(&self) -> bool {
        match self {
            MissionToken::Time(_) => false,
            MissionToken::Trust(_) => true,
            MissionToken::Energy(_) => true,
            MissionToken::Labor(_) => true,
        }
    }
    fn class_id(&self) -> u64 {
        0
    }
    fn asset_id(&self) -> u64 {
        0
    }
    fn drag(&mut self) -> bool {
        false
    }
    fn drop(&mut self, _src_class_id: u64, _src_asset_id: u64, _source_amount: Amount) -> bool {
       match self {
            MissionToken::Time(asset) => asset.0 = _source_amount,
            MissionToken::Trust(asset) => asset.0 = _source_amount,
            MissionToken::Energy(asset) => asset.0 = _source_amount,
            MissionToken::Labor(asset) => asset.0 = _source_amount,
        };
        false
    }
    fn push_as_children(&self, _commands: &mut Commands, _parent: Entity) {}
}