use crate::actions::*;
use crate::asset::{BTNode, BehaviorAsset, BehaviorAssetLoader, BehaviorDocument};
use crate::composites::*;
use crate::{gen_tree, BehaviorCursor, BehaviorInfo, BehaviorRunning, BehaviorSpawner};
use bevy::{ecs::system::EntityCommands, prelude::*, reflect::TypeUuid};
use serde::{Deserialize, Serialize};

pub struct MyBehaviorPlugin;

impl Plugin for MyBehaviorPlugin {
    fn build(&self, app: &mut App) {
        app.add_asset::<BehaviorAsset<MyBehavior>>()
            .init_asset_loader::<BehaviorAssetLoader<MyBehavior>>()
            .add_startup_system(data_test)
            .add_system(behevoir_document_test);
    }
}

#[derive(Serialize, Deserialize, TypeUuid)]
#[uuid = "5c3fbd4c-5359-11ed-9c5d-02a179e5df2b"]
pub enum MyBehavior {
    DebugAction(DebugAction),
    Selector(Selector),
    Sequence(Sequence),
}

impl Default for MyBehavior {
    fn default() -> Self {
        Self::DebugAction(DebugAction::default())
    }
}

impl BehaviorSpawner for MyBehavior {
    fn spawn_with(&self, commands: &mut EntityCommands) {
        match self {
            MyBehavior::DebugAction(action) => BehaviorInfo::spawn_with(commands, action),
            MyBehavior::Selector(selector) => BehaviorInfo::spawn_with(commands, selector),
            MyBehavior::Sequence(sequence) => BehaviorInfo::spawn_with(commands, sequence),
        }
    }
}

fn behevoir_document_test(
    mut commands: Commands,
    mut document: Local<Option<Handle<BehaviorAsset<MyBehavior>>>>,
    mut behavior: Local<Option<Entity>>,
    asset_server: Res<AssetServer>,
    bhts: Res<Assets<BehaviorAsset<MyBehavior>>>,
) {
    if document.is_none() {
        *document = Some(asset_server.load("behaviors/debug_test.bht.ron"));
    }

    if behavior.is_none() {
        if let Some(document) = &*document {
            if let Some(behavior_asset) = bhts.get(&document) {
                println!("behavior_asset LOADED");
                let root_entity = gen_tree(None, &mut commands, &behavior_asset.document.root);
                commands
                    .entity(root_entity)
                    .insert(BehaviorRunning)
                    .insert(BehaviorCursor);
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
                MyBehavior::Sequence(Sequence::default()),
                vec![
                    BTNode(
                        MyBehavior::DebugAction(DebugAction {
                            message: "Hello, from DebugMessage 0!".to_string(),
                            ..default()
                        }),
                        vec![],
                    ),
                    BTNode(
                        MyBehavior::DebugAction(DebugAction {
                            message: "Hello, from DebugMessage 1!".to_string(),
                            repeat: 5,
                            ..default()
                        }),
                        vec![],
                    ),
                ],
            ),
        },
    };
    let data_str = ron::to_string(&data.document).unwrap();
    println!("{}", data_str);
    let data = ron::from_str::<BehaviorDocument<MyBehavior>>(&data_str);
    assert!(data.is_ok());
}
