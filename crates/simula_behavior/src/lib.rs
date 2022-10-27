use actions::*;
use asset::{BTNode, BehaviorAssetLoading};
use bevy::{ecs::query::WorldQuery, ecs::system::EntityCommands, prelude::*, reflect::TypeUuid};
use bevy_inspector_egui::Inspectable;
use composites::*;
use decorators::*;
use std::fmt::Debug;

pub use crate::asset::{BehaviorAsset, BehaviorDocument};

pub mod actions;
pub mod asset;
pub mod composites;
pub mod decorators;
pub mod editor;
pub mod test;

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

/// How to spawn a behavior node
pub trait BehaviorSpawner {
    fn spawn_with(&self, commands: &mut EntityCommands);
}

impl Plugin for BehaviorPlugin {
    fn build(&self, app: &mut App) {
        app.register_type::<editor::BehaviorGraphState>()
            .register_type::<editor::BehaviorEditorState>()
            .register_type::<BehaviorSuccess>()
            .register_type::<BehaviorRunning>()
            .register_type::<BehaviorFailure>()
            .add_system(editor::egui_update)
            .add_system(complete_behavior)
            .add_system(start_behavior)
            .add_system(sequence::run)
            .add_system(selector::run)
            .add_system(repeater::run)
            .add_system(inverter::run)
            .add_system(succeeder::run)
            .add_system(delay::run)
            .add_system(debug_action::run);
    }
}

/// A marker added to entities that want to run a behavior
#[derive(Debug, Default, Reflect, Clone, Copy, Component, Inspectable, PartialEq)]
#[reflect(Component)]
#[component(storage = "SparseSet")]
pub struct BehaviorRunning {
    pub on_enter_handled: bool,
}

/// A marker added to behaviors that complete with success
#[derive(Debug, Default, Reflect, Clone, Copy, Component, Inspectable, PartialEq)]
#[reflect(Component)]
#[component(storage = "SparseSet")]
pub struct BehaviorSuccess;

/// A marker added to behaviors that complete with failure
#[derive(Debug, Default, Reflect, Clone, Copy, Component, Inspectable, PartialEq)]
#[reflect(Component)]
#[component(storage = "SparseSet")]
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
    Composite,
    Decorator,
}

/// A component to provide static behavior node info
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

impl BehaviorTree {
    /// Spawn from behavior asset
    pub fn from_asset<T>(
        parent: Option<Entity>,
        commands: &mut Commands,
        document: Handle<BehaviorAsset<T>>,
    ) -> Self
    where
        T: TypeUuid + Send + Sync + 'static + Default + Debug,
    {
        let entity = commands
            .spawn()
            .insert(BehaviorAssetLoading { document, parent })
            .id();
        Self { root: Some(entity) }
    }

    /// Spawn a behavior tree from a document, and return a BehaviorTree component with tree root
    pub fn from_document<T>(
        parent: Option<Entity>,
        commands: &mut Commands,
        document: &BehaviorDocument<T>,
    ) -> Self
    where
        T: Default + BehaviorSpawner,
    {
        let entity = Self::spawn_tree(parent, commands, &document.root);
        Self { root: Some(entity) }
    }

    /// Spawn a behavior tree from a behavior node, and return a BehaviorTree component with tree root
    pub fn from_node<T>(parent: Option<Entity>, commands: &mut Commands, node: &BTNode<T>) -> Self
    where
        T: Default + BehaviorSpawner,
    {
        let entity = Self::spawn_tree(parent, commands, node);
        Self { root: Some(entity) }
    }

    /// Spawn a behavior tree from a behavior node
    pub fn insert_tree<T>(
        entity: Entity,
        parent: Option<Entity>,
        commands: &mut Commands,
        node: &BTNode<T>,
    ) -> Entity
    where
        T: Default + BehaviorSpawner,
    {
        let BTNode(node_type, nodes) = node;
        let mut entity_commands = commands.entity(entity);
        node_type.spawn_with(&mut entity_commands);
        entity_commands.insert(BehaviorParent(parent));
        let children = nodes
            .iter()
            .map(|node| Self::insert_tree(commands.spawn().id(), Some(entity), commands, node))
            .collect::<Vec<Entity>>();
        add_children(commands, entity, &children);
        entity
    }

    /// Spawn a behavior tree from a behavior node
    pub fn spawn_tree<T>(
        parent: Option<Entity>,
        commands: &mut Commands,
        node: &BTNode<T>,
    ) -> Entity
    where
        T: Default + BehaviorSpawner,
    {
        let entity = commands.spawn().id();
        Self::insert_tree(entity, parent, commands, node);
        entity
    }
}

/// A marker added to currently running behaviors
#[derive(Default, Debug, Reflect, Clone, Component, Inspectable)]
#[reflect(Component)]
#[component(storage = "SparseSet")]
pub struct BehaviorCursor;

/// Query filter for running behaviors
#[derive(WorldQuery)]
pub struct BehaviorRunQuery {
    _node: With<BehaviorNode>,
    _cursor: With<BehaviorCursor>,
    _running: With<BehaviorRunning>,
    _failure: Without<BehaviorFailure>,
    _success: Without<BehaviorSuccess>,
}

/// Query filter for behaviors ready to start
#[derive(WorldQuery)]
pub struct BehaviorReadyQuery {
    _node: With<BehaviorNode>,
    _cursor: With<BehaviorCursor>,
    _running: Without<BehaviorRunning>,
    _failure: Without<BehaviorFailure>,
    _success: Without<BehaviorSuccess>,
}

/// Query filter for behaviors that have completed
#[derive(WorldQuery)]
pub struct BehaviorDoneQuery {
    _node: With<BehaviorNode>,
    _cursor: With<BehaviorCursor>,
    _running: With<BehaviorRunning>,
    _done: Or<(With<BehaviorFailure>, With<BehaviorSuccess>)>,
}

/// Query for behavior children
#[derive(WorldQuery)]
pub struct BehaviorChildQuery {
    child_entity: Entity,
    child_parent: &'static BehaviorParent,
    child_failure: Option<&'static BehaviorFailure>,
    child_success: Option<&'static BehaviorSuccess>,
}

#[derive(WorldQuery)]
pub struct BehaviorChildQueryFilter {
    _node: With<BehaviorNode>,
    _cursor: Without<BehaviorCursor>,
    _running: Without<BehaviorRunning>,
}

#[derive(Default, Debug, Clone, Deref, DerefMut, PartialEq)]
pub struct BehaviorTrace(pub Vec<String>);
impl BehaviorTrace {
    pub fn from_list(traces: &[&str]) -> Self {
        Self(traces.iter().map(|s| s.to_string()).collect())
    }
}

/// Process completed behaviors, pass cursor to parent
pub fn complete_behavior(
    mut commands: Commands,
    mut dones: Query<
        (
            Entity,
            Option<&BehaviorSuccess>,
            Option<&BehaviorFailure>,
            &BehaviorParent,
            &Name,
        ),
        BehaviorDoneQuery,
    >,
    mut trace: Option<ResMut<BehaviorTrace>>,
) {
    for (entity, success, failure, parent, name) in dones.iter_mut() {
        let state = if success.is_some() {
            "SUCCESS"
        } else if failure.is_some() {
            "FAILURE"
        } else {
            error!("Behavior is both success and failure");
            "ERROR"
        };
        debug!(
            "[{}] {} {}",
            entity.id().to_string(),
            state,
            name.to_string()
        );
        if let Some(trace) = trace.as_mut() {
            trace.push(format!("[{}] {} {}", entity.id(), state, name.to_string(),));
        }
        commands.entity(entity).remove::<BehaviorRunning>();
        commands.entity(entity).remove::<BehaviorCursor>();
        if let Some(parent) = **parent {
            commands.entity(parent).insert(BehaviorCursor);
        }
    }
}

/// Process ready behaviors, start them
pub fn start_behavior(
    mut commands: Commands,
    mut ready: Query<(Entity, &BehaviorChildren, &Name), BehaviorReadyQuery>,
    nodes: Query<Entity, (With<BehaviorNode>, Without<BehaviorCursor>)>,
    mut trace: Option<ResMut<BehaviorTrace>>,
) {
    for (entity, children, name) in ready.iter_mut() {
        // Reset all children
        // debug!("[{}] RESETNG {}", entity.id(), name.to_string());
        for entity in nodes.iter_many(children.iter()) {
            commands.entity(entity).remove::<BehaviorRunning>();
            commands.entity(entity).remove::<BehaviorSuccess>();
            commands.entity(entity).remove::<BehaviorFailure>();
        }
        debug!("[{}] STARTED {}", entity.id().to_string(), name.to_string());
        if let Some(trace) = trace.as_mut() {
            trace.push(format!("[{}] STARTED {}", entity.id(), name.to_string(),));
        }
        commands.entity(entity).insert(BehaviorRunning::default());
    }
}
