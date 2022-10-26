use crate::behaviors;
use bevy::{ecs::system::EntityCommands, prelude::*, reflect::TypeUuid};
use serde::{Deserialize, Serialize};
use simula_behavior::{
    actions::*,
    asset::{async_loader, BTNode, BehaviorAsset, BehaviorAssetLoader, BehaviorDocument},
    composites::*,
    decorators::*,
    BehaviorCursor, BehaviorInfo, BehaviorSpawner, BehaviorTree,
};

pub struct MissionBehaviorPlugin;

impl Plugin for MissionBehaviorPlugin {
    fn build(&self, app: &mut App) {
        app.add_startup_system(setup)
            .add_asset::<BehaviorAsset<MissionBehavior>>()
            .init_asset_loader::<BehaviorAssetLoader<MissionBehavior>>()
            .add_system(async_loader::<MissionBehavior>)
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

fn setup() {}

fn create_from_data() {
    let document = BehaviorDocument {
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
        };
    let data_str = ron::to_string(&document).unwrap();
    println!("{}", data_str);
    let data = ron::from_str::<BehaviorDocument<MissionBehavior>>(&data_str);
    assert!(data.is_ok());
    // let data = data.unwrap();
    // println!("{:#?}", data);
}

fn create_from_asset(
    mut commands: Commands,
    mut document: Local<Option<Handle<BehaviorAsset<MissionBehavior>>>>,
    mut behavior: Local<Option<Entity>>,
    asset_server: Res<AssetServer>,
    bhts: Res<Assets<BehaviorAsset<MissionBehavior>>>,
) {
    // if document.is_none() {
    //     *document = Some(asset_server.load("behaviors/debug_repeater.bht.ron"));
    // }
    // if behavior.is_none() {
    //     if let Some(document) = &*document {
    //         if let Some(behavior_asset) = bhts.get(&document) {
    //             let root_entity = commands.spawn().id();
    //             let root_entity = BehaviorTree::insert_tree(
    //                 root_entity,
    //                 None,
    //                 &mut commands,
    //                 &behavior_asset.document.root,
    //             );
    //             commands.entity(root_entity).insert(BehaviorCursor);
    //             *behavior = Some(root_entity);
    //         }
    //     }
    // }
}


