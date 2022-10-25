use crate::behaviors;
use bevy::{ecs::system::EntityCommands, prelude::*, reflect::TypeUuid};
use serde::{Deserialize, Serialize};
use simula_behavior::{
    actions::*,
    asset::{BTNode, BehaviorAsset, BehaviorAssetLoader, BehaviorDocument},
    composites::*,
    decorators::*,
    spawn_tree, BehaviorCursor, BehaviorInfo, BehaviorSpawner,
};

pub struct MissionBehaviorPlugin;

impl Plugin for MissionBehaviorPlugin {
    fn build(&self, app: &mut App) {
        app.add_asset::<BehaviorAsset<MissionBehavior>>()
            .init_asset_loader::<BehaviorAssetLoader<MissionBehavior>>()
            .add_startup_system(data_test)
            .add_system(behevoir_document_test)
            .add_system(behaviors::agent_rest::run)
            .add_system(behaviors::agent_work::run);
    }
}

#[derive(Debug, Serialize, Deserialize, TypeUuid)]
#[uuid = "5c3fbd4c-5359-11ed-9c5d-02a179e5df2b"]
pub enum MissionBehavior {
    DebugAction(DebugAction),
    Selector(Selector),
    Sequence(Sequence),
    Repeater(Repeater),
    AgentRest(behaviors::agent_rest::AgentRest),
    AgentWork(behaviors::agent_work::AgentWork),
}

impl Default for MissionBehavior {
    fn default() -> Self {
        Self::DebugAction(DebugAction::default())
    }
}

impl BehaviorSpawner for MissionBehavior {
    fn spawn_with(&self, commands: &mut EntityCommands) {
        match self {
            MissionBehavior::DebugAction(action) => BehaviorInfo::spawn_with(commands, action),
            MissionBehavior::Selector(selector) => BehaviorInfo::spawn_with(commands, selector),
            MissionBehavior::Sequence(sequence) => BehaviorInfo::spawn_with(commands, sequence),
            MissionBehavior::Repeater(repeater) => BehaviorInfo::spawn_with(commands, repeater),
            MissionBehavior::AgentRest(agent) => BehaviorInfo::spawn_with(commands, agent),
            MissionBehavior::AgentWork(agent) => BehaviorInfo::spawn_with(commands, agent),
        }
    }
}

fn behevoir_document_test(
    mut commands: Commands,
    mut document: Local<Option<Handle<BehaviorAsset<MissionBehavior>>>>,
    mut behavior: Local<Option<Entity>>,
    asset_server: Res<AssetServer>,
    bhts: Res<Assets<BehaviorAsset<MissionBehavior>>>,
) {
    if document.is_none() {
        *document = Some(asset_server.load("behaviors/debug_repeater.bht.ron"));
    }
    if behavior.is_none() {
        if let Some(document) = &*document {
            if let Some(behavior_asset) = bhts.get(&document) {
                println!("behavior_asset LOADED");
                let root_entity = spawn_tree(None, &mut commands, &behavior_asset.document.root);
                commands.entity(root_entity).insert(BehaviorCursor);
                *behavior = Some(root_entity);
            }
        }
    }
}

fn data_test() {
    let data = BehaviorAsset {
        path: "test".to_string(),
        document: BehaviorDocument {
            root: BTNode(
                MissionBehavior::Repeater(Repeater {
                    repeat: Repeat::Times(2),
                    ..default()
                }),
                vec![BTNode(
                    MissionBehavior::Sequence(Sequence::default()),
                    vec![
                        BTNode(
                            MissionBehavior::DebugAction(DebugAction {
                                message: "Hello, from DebugMessage0!".to_string(),
                                ..default()
                            }),
                            vec![],
                        ),
                        BTNode(
                            MissionBehavior::DebugAction(DebugAction {
                                message: "Hello, from DebugMessage1!".to_string(),
                                repeat: 5,
                                ..default()
                            }),
                            vec![],
                        ),
                    ],
                )],
            ),
        },
    };
    let data_str = ron::to_string(&data.document).unwrap();
    println!("{}", data_str);
    let data = ron::from_str::<BehaviorDocument<MissionBehavior>>(&data_str);
    assert!(data.is_ok());
    // let data = data.unwrap();
    // println!("{:#?}", data);
}
