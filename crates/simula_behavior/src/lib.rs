use actions::*;
use asset::{BTNode, BehaviorAsset, BehaviorAssetLoader};
use bevy::{ecs::query::WorldQuery, ecs::system::EntityCommands, prelude::*, reflect::TypeUuid};
use bevy_inspector_egui::Inspectable;
use composites::*;
use serde::{Deserialize, Serialize};

pub use crate::asset::BehaviorDocument;

pub mod actions;
pub mod asset;
pub mod composites;
pub mod decorators;
pub mod editor;

pub struct BehaviorPlugin;

#[derive(Bundle)]
pub struct BehaviorBundle<T>
where
    T: Reflect + Component + Clone,
{
    pub behavior: T,
    pub node: BehaviorNode,
    pub typ: BehaviorType,
    pub name: Name,
    pub parent: BehaviorParent,
    pub children: BehaviorChildren,
}

impl<T> Default for BehaviorBundle<T>
where
    T: BehaviorInfo + Default + Reflect + Component + Clone,
{
    fn default() -> Self {
        Self {
            behavior: T::default(),
            node: BehaviorNode::default(),
            typ: T::TYPE,
            name: Name::new(std::any::type_name::<T>()),
            parent: BehaviorParent::default(),
            children: BehaviorChildren::default(),
        }
    }
}

#[derive(Serialize, Deserialize, TypeUuid)]
#[uuid = "5c3fbd4c-5359-11ed-9c5d-02a179e5df2b"]
pub enum DebugNode {
    DebugAction(DebugAction),
    Selector(Selector),
    Sequence(Sequence),
}

impl Default for DebugNode {
    fn default() -> Self {
        Self::DebugAction(DebugAction::default())
    }
}

trait BehaviorNodeInfo {
    fn spawn_with(&self, commands: &mut EntityCommands);
}

impl BehaviorNodeInfo for DebugNode {
    fn spawn_with(&self, commands: &mut EntityCommands) {
        match self {
            DebugNode::DebugAction(action) => BehaviorInfo::spawn_with(commands, action),
            DebugNode::Selector(selector) => BehaviorInfo::spawn_with(commands, selector),
            DebugNode::Sequence(sequence) => BehaviorInfo::spawn_with(commands, sequence),
        }
    }
}

impl Plugin for BehaviorPlugin {
    fn build(&self, app: &mut App) {
        app.add_asset::<BehaviorAsset<DebugNode>>()
            .init_asset_loader::<BehaviorAssetLoader<DebugNode>>()
            .register_type::<editor::BehaviorGraphState>()
            .register_type::<editor::BehaviorEditorState>()
            .register_type::<BehaviorSuccess>()
            .register_type::<BehaviorRunning>()
            .register_type::<BehaviorFailure>()
            .add_startup_system(data_test)
            .add_system(behevoir_document_test)
            .add_system(editor::egui_update)
            .add_system(complete_behavior)
            .add_system(sequence::run)
            .add_system(selector::run)
            .add_system(debug_action::run);
    }
}

fn gen_tree<T>(parent: Option<Entity>, commands: &mut Commands, node: &BTNode<T>) -> Entity
where
    T: Default + BehaviorNodeInfo,
{
    let BTNode(node_type, nodes) = node;
    let mut entity = commands.spawn();
    node_type.spawn_with(&mut entity);
    let entity = entity.insert(BehaviorParent(parent)).id();

    if parent.is_none() {
        commands
            .entity(entity)
            .insert(BehaviorRunning)
            .insert(BehaviorCursor);
    }

    let children = nodes
        .iter()
        .map(|node| gen_tree(Some(entity), commands, node))
        .collect::<Vec<Entity>>();

    add_children(commands, entity, &children);

    entity
}

fn behevoir_document_test(
    mut commands: Commands,
    mut document: Local<Option<Handle<BehaviorAsset<DebugNode>>>>,
    mut behavior: Local<Option<Entity>>,
    asset_server: Res<AssetServer>,
    bhts: Res<Assets<BehaviorAsset<DebugNode>>>,
) {
    if document.is_none() {
        *document = Some(asset_server.load("behaviors/debug_test.bht.ron"));
    }

    if behavior.is_none() {
        if let Some(document) = &*document {
            if let Some(behavior_asset) = bhts.get(&document) {
                println!("behavior_asset LOADED");
                *behavior = Some(gen_tree(None, &mut commands, &behavior_asset.document.root));
            }
        }
    }
}

fn data_test() {
    let data = BehaviorAsset {
        path: "test".to_string(),
        document: BehaviorDocument {
            root: BTNode(
                DebugNode::Sequence(Sequence::default()),
                vec![
                    BTNode(
                        DebugNode::DebugAction(DebugAction {
                            message: "Hello, from DebugMessage 0!".to_string(),
                            ..default()
                        }),
                        vec![],
                    ),
                    BTNode(
                        DebugNode::DebugAction(DebugAction {
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
    let data = ron::from_str::<BehaviorDocument<DebugNode>>(&data_str);
    assert!(data.is_ok());
}

/// A marker added to entities that want to run a behavior
#[derive(Debug, Default, Reflect, Clone, Copy, Component, Inspectable, PartialEq)]
#[reflect(Component)]
pub struct BehaviorRunning;

/// A marker added to behaviors that complete with success
#[derive(Debug, Default, Reflect, Clone, Copy, Component, Inspectable, PartialEq)]
#[reflect(Component)]
pub struct BehaviorSuccess;

/// A marker added to behaviors that complete with failure
#[derive(Debug, Default, Reflect, Clone, Copy, Component, Inspectable, PartialEq)]
#[reflect(Component)]
pub struct BehaviorFailure;

/// A marker added to behavior node entities
#[derive(Debug, Default, Reflect, Clone, Component, Inspectable)]
#[reflect(Component)]
pub struct BehaviorNode;

/// A component to point to the parent of a behavior node
#[derive(Deref, Debug, Default, Reflect, Clone, Component)]
#[reflect(Component)]
pub struct BehaviorParent(Option<Entity>);

/// A component to point to the children of a behavior node
#[derive(Deref, Debug, Default, Reflect, Clone, Component)]
#[reflect(Component)]
pub struct BehaviorChildren(Vec<Entity>);

/// A component added to identify the type of a behavior node
#[derive(Debug, Default, Reflect, Clone, Component, Inspectable)]
#[reflect(Component)]
pub enum BehaviorType {
    #[default]
    Action,
    Sequence,
    Selector,
}

pub trait BehaviorInfo
where
    Self: Reflect + Component + Clone + Default + Sized + 'static,
{
    const TYPE: BehaviorType;
    const NAME: &'static str;
    const DESC: &'static str;

    fn spawn(commands: &mut EntityCommands) {
        commands.insert_bundle(BehaviorBundle::<Self>::default());
    }

    fn spawn_with(commands: &mut EntityCommands, data: &Self) {
        commands.insert_bundle(BehaviorBundle::<Self> {
            behavior: data.clone(),
            ..Default::default()
        });
    }
}

pub fn add_children(commands: &mut Commands, parent: Entity, children: &[Entity]) {
    commands
        .entity(parent)
        .insert(BehaviorChildren(children.iter().copied().collect()));
    for child in children {
        commands.entity(*child).insert(BehaviorParent(Some(parent)));
    }
    commands.entity(parent).push_children(children);
}

/// A component added to identify the root of a behavior tree
#[derive(Default, Reflect, Clone, Component, Inspectable)]
#[reflect(Component)]
pub struct BehaviorTree {
    pub root: Option<Entity>,
}

/// A marker added to currently running behaviors
#[derive(Default, Reflect, Clone, Component, Inspectable)]
#[reflect(Component)]
pub struct BehaviorCursor;

/// Query filter for running behaviors
#[derive(WorldQuery)]
pub struct BehaviorRunQuery {
    _cursor: With<BehaviorCursor>,
    _running: With<BehaviorRunning>,
    _failure: Without<BehaviorFailure>,
    _success: Without<BehaviorSuccess>,
}

/// Query filter for behaviors ready to run
#[derive(WorldQuery)]
pub struct BehaviorWithoutQuery {
    _cursor: Without<BehaviorCursor>,
    _running: Without<BehaviorRunning>,
    _failure: Without<BehaviorFailure>,
    _success: Without<BehaviorSuccess>,
}

/// Query filter for behaviors that have completed
#[derive(WorldQuery)]
pub struct BehaviorDoneQuery {
    _cursor: With<BehaviorCursor>,
    _running: With<BehaviorRunning>,
    _done: Or<(With<BehaviorFailure>, With<BehaviorSuccess>)>,
}

/// Process completed behaviors, pass cursor to parent
pub fn complete_behavior(
    mut commands: Commands,
    mut dones: Query<(Entity, &BehaviorParent, &Name), BehaviorDoneQuery>,
) {
    for (entity, parent, name) in dones.iter_mut() {
        debug!("{} is done", name.to_string());
        commands.entity(entity).remove::<BehaviorRunning>();
        commands.entity(entity).remove::<BehaviorCursor>();
        if let Some(parent) = **parent {
            commands.entity(parent).insert(BehaviorCursor);
        }
    }
}
