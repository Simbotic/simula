use actions::*;
use asset::{
    behavior_document_to_asset, behavior_tree_reset, Behavior, BehaviorAsset, BehaviorAssetLoader,
    BehaviorDocument,
};
use bevy::{
    ecs::{
        entity::{EntityMap, MapEntities, MapEntitiesError},
        query::WorldQuery,
        reflect::ReflectMapEntities,
        system::EntityCommands,
    },
    prelude::*,
    reflect::{TypeRegistry, TypeUuid},
};
use composites::*;
use decorators::*;
use serde::{Deserialize, Serialize};
use simula_script::{ScriptContext, ScriptPlugin};
use strum::AsRefStr;

pub mod actions;
pub mod asset;
pub mod composites;
pub mod decorators;
pub mod inspector;
pub mod property;
pub mod protocol;
pub mod server;
pub mod test;

pub mod prelude {
    pub use crate::actions::*;
    pub use crate::asset::{
        Behavior, BehaviorAsset, BehaviorAssetLoader, BehaviorDocument, BehaviorTreeReset,
    };
    pub use crate::composites::*;
    pub use crate::decorators::*;
    pub use crate::inspector::{
        BehaviorInspectable, BehaviorInspectorPlugin, BehaviorNodeInspectable, BehaviorPropEPath,
        BehaviorPropStr, BehaviorUI,
    };
    pub use crate::property::{BehaviorEval, BehaviorProp, BehaviorPropGeneric, BehaviorPropValue};
    pub use crate::protocol::{self};
    pub use crate::server::{
        AssetTracker, BehaviorServerPlugin, BehaviorTracker, BehaviorTrackers, EntityTracker,
    };
    pub use crate::{
        BehaviorChildQuery, BehaviorChildQueryFilter, BehaviorChildQueryItem, BehaviorChildren,
        BehaviorCursor, BehaviorFactory, BehaviorFailure, BehaviorIdleQuery, BehaviorMissing,
        BehaviorNode, BehaviorParent, BehaviorPlugin, BehaviorRunQuery, BehaviorRunning,
        BehaviorSet, BehaviorSpec, BehaviorStarted, BehaviorSuccess, BehaviorTree,
        BehaviorTreePlugin, BehaviorType,
    };
}

pub struct BehaviorPlugin;

impl Plugin for BehaviorPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugin(ScriptPlugin)
            .init_asset_loader::<BehaviorAssetLoader>()
            .add_asset::<BehaviorDocument>()
            .configure_set(BehaviorSet::PostUpdate.in_base_set(CoreSet::PostUpdate))
            .add_systems(
                (clear_behavior_started, complete_behavior, start_behavior)
                    .chain()
                    .in_set(BehaviorSet::PostUpdate),
            )
            .register_type::<BehaviorNode>()
            .register_type::<BehaviorSuccess>()
            .register_type::<BehaviorRunning>()
            .register_type::<BehaviorFailure>()
            .register_type::<BehaviorCursor>()
            .register_type::<BehaviorParent>()
            .register_type::<BehaviorChildren>()
            .register_type::<BehaviorType>()
            .register_type::<Debug>()
            .register_type::<Selector>()
            .register_type::<Sequencer>()
            .register_type::<All>()
            .register_type::<Any>()
            .register_type::<Repeater>()
            .register_type::<Inverter>()
            .register_type::<Succeeder>()
            .register_type::<Wait>()
            .register_type::<Delay>()
            .register_type::<Identity>()
            .register_type::<Guard>()
            .register_type::<Timeout>()
            .add_system(debug::run)
            .add_system(selector::run)
            .add_system(sequencer::run)
            .add_system(all::run)
            .add_system(any::run)
            .add_system(repeater::run)
            .add_system(inverter::run)
            .add_system(succeeder::run)
            .add_system(wait::run)
            .add_system(delay::run)
            .add_system(identity::run)
            .add_system(guard::run)
            .add_system(timeout::run);
    }
}

#[derive(SystemSet, Debug, Clone, PartialEq, Eq, Hash)]
pub enum BehaviorSet {
    PostUpdate,
}

#[derive(Default)]
pub struct BehaviorTreePlugin<T: BehaviorFactory>(pub std::marker::PhantomData<T>);

impl<T> Plugin for BehaviorTreePlugin<T>
where
    T: BehaviorFactory + Serialize + for<'de> Deserialize<'de>,
{
    fn build(&self, app: &mut App) {
        app.register_type::<BehaviorTree<T>>()
            .add_asset::<BehaviorAsset<T>>()
            .add_systems((behavior_document_to_asset::<T>, behavior_tree_reset::<T>).chain());
    }
}

pub struct BehaviorMissing;

/// How to spawn a behavior node
pub trait BehaviorFactory:
    Clone
    + Default
    + TypeUuid
    + Send
    + Sync
    + 'static
    + Default
    + std::fmt::Debug
    + Reflect
    + FromReflect
{
    type Attributes: Clone
        + Default
        + Send
        + Sync
        + 'static
        + Default
        + std::fmt::Debug
        + Serialize
        + for<'de> Deserialize<'de>;

    /// insert behavior components into the entity
    fn insert(&self, commands: &mut EntityCommands);

    /// get behavior label
    fn label(&self) -> &str;

    /// get behavior icon
    fn icon(&self) -> &str;

    /// get behavior description
    fn desc(&self) -> &str;

    /// get behavior type: composite, decorator, action
    fn typ(&self) -> BehaviorType;

    /// get behavior properties for inspector
    fn inner_reflect(&self) -> &dyn Reflect;

    /// get mutable behavior properties for inspector
    fn inner_reflect_mut(&mut self) -> &mut dyn Reflect;

    /// ui inspector for behavior properties
    fn ui(
        &mut self,
        state: Option<protocol::BehaviorState>,
        ui: &mut bevy_inspector_egui::egui::Ui,
        type_registry: &TypeRegistry,
    ) -> bool;

    /// ui readonly inspector for behavior properties
    fn ui_readonly(
        &self,
        state: Option<protocol::BehaviorState>,
        ui: &mut bevy_inspector_egui::egui::Ui,
        type_registry: &TypeRegistry,
    );

    /// copy behavior data from entity into this behavior
    fn copy_from(&mut self, _entity: Entity, _world: &World) -> Result<(), BehaviorMissing>;

    /// list all behaviors, with an instance of each
    fn list() -> Vec<Self>;
}

/// A marker added to currently running behaviors
#[derive(Default, Debug, Reflect, Clone, Copy, Component)]
#[reflect(Component)]
#[component(storage = "SparseSet")]
pub enum BehaviorCursor {
    /// Delegate execution to child
    #[default]
    Delegate,
    /// Return execution to parent
    Return,
}

/// A marker added to entities that want to run a behavior
#[derive(Debug, Default, Reflect, Clone, Copy, Component, PartialEq)]
#[reflect(Component)]
#[component(storage = "SparseSet")]
pub struct BehaviorRunning;

/// A marker added to entities that want to run a behavior
#[derive(Debug, Default, Reflect, Clone, Copy, Component, PartialEq)]
#[reflect(Component)]
#[component(storage = "SparseSet")]
pub struct BehaviorPaused;

/// A marker added to entities that want to run a behavior
#[derive(Debug, Default, Reflect, Clone, Copy, Component, PartialEq)]
#[reflect(Component)]
#[component(storage = "SparseSet")]
pub struct BehaviorStarted;

/// A marker added to behaviors that complete with success
#[derive(Debug, Default, Reflect, Clone, Copy, Component, PartialEq)]
#[reflect(Component)]
#[component(storage = "SparseSet")]
pub struct BehaviorSuccess;

/// A marker added to behaviors that complete with failure
#[derive(Debug, Default, Reflect, Clone, Copy, Component, PartialEq)]
#[reflect(Component)]
#[component(storage = "SparseSet")]
pub struct BehaviorFailure;

/// A marker added to behaviors that are stopped without completing
#[derive(Debug, Default, Reflect, Clone, Copy, Component, PartialEq)]
#[reflect(Component)]
#[component(storage = "SparseSet")]
pub struct BehaviorStopped;

/// A marker added to behavior node entities
#[derive(Component, Debug, Eq, PartialEq, Reflect)]
#[reflect(Component, MapEntities, PartialEq)]
pub struct BehaviorNode {
    pub tree: Entity,
}

impl FromWorld for BehaviorNode {
    fn from_world(_world: &mut World) -> Self {
        BehaviorNode {
            tree: Entity::PLACEHOLDER,
        }
    }
}

impl MapEntities for BehaviorNode {
    fn map_entities(&mut self, entity_map: &EntityMap) -> Result<(), MapEntitiesError> {
        if let Ok(mapped_entity) = entity_map.get(self.tree) {
            self.tree = mapped_entity;
        }
        Ok(())
    }
}

/// A component to point to the parent of a behavior node
#[derive(Component, Debug, Eq, PartialEq, Reflect, Deref, DerefMut)]
#[reflect(Component, MapEntities, PartialEq)]
pub struct BehaviorParent(Entity);

impl FromWorld for BehaviorParent {
    fn from_world(_world: &mut World) -> Self {
        BehaviorParent(Entity::PLACEHOLDER)
    }
}

impl MapEntities for BehaviorParent {
    fn map_entities(&mut self, entity_map: &EntityMap) -> Result<(), MapEntitiesError> {
        if let Ok(mapped_entity) = entity_map.get(self.0) {
            self.0 = mapped_entity;
        }
        Ok(())
    }
}

/// A component to point to the children of a behavior node
#[derive(Deref, DerefMut, Debug, Default, Reflect, Clone, Component)]
#[reflect(Component)]
pub struct BehaviorChildren(Vec<Entity>);

/// A component added to identify the type of a behavior node
#[derive(Debug, Default, PartialEq, Reflect, Clone, Component, Copy, AsRefStr)]
#[reflect(Component)]
pub enum BehaviorType {
    #[default]
    Action,
    Composite,
    Decorator,
    Subtree,
}

/// A component to provide static behavior node definition
pub trait BehaviorSpec
where
    Self: Reflect + Component + Clone + Default + Sized + 'static,
{
    const TYPE: BehaviorType;
    const NAME: &'static str;
    const ICON: &'static str;
    const DESC: &'static str;

    fn insert_with(commands: &mut EntityCommands, data: &Self) {
        commands.insert(data.clone());
    }
}

/// A component added to identify the root of a behavior tree
#[derive(Default, Reflect, Clone, Component)]
#[reflect(Component)]
pub struct BehaviorTree<T: BehaviorFactory>(#[reflect(ignore)] std::marker::PhantomData<T>);

impl<T> BehaviorTree<T>
where
    T: BehaviorFactory,
{
    /// Create a script context to be used by the behavior tree
    pub fn create_script_context() -> ScriptContext {
        let mut scope = ScriptContext::new();
        let mut blackboard = simula_script::script::Map::new();
        blackboard.insert("state".into(), 0.into());
        scope.scope.push("blackboard", blackboard);
        scope
    }

    /// Build behavior tree from a behavior node.
    fn insert_tree(
        tree: Entity,
        parent: Option<Entity>,
        commands: &mut Commands,
        node: &Behavior<T>,
    ) -> Entity {
        let entity = commands.spawn_empty().id();

        let mut entity_commands = commands.entity(entity);
        node.data().insert(&mut entity_commands);
        entity_commands.insert(Name::new(node.name().to_owned()));
        if let Some(parent) = parent {
            entity_commands.insert(BehaviorParent(parent));
        }
        entity_commands.insert(BehaviorNode { tree });

        let children = node
            .nodes()
            .iter()
            .map(|node| Self::insert_tree(tree, Some(entity), commands, node))
            .collect::<Vec<Entity>>();
        Self::add_children(commands, entity, &children);
        entity
    }

    /// Add chidren to a behavior node using bevy hierarchy and behavior hierarchy
    fn add_children(commands: &mut Commands, parent: Entity, children: &[Entity]) {
        if children.is_empty() {
            return;
        }
        commands
            .entity(parent)
            .insert(BehaviorChildren(children.to_vec()));
        for child in children {
            commands.entity(*child).insert(BehaviorParent(parent));
        }
        commands.entity(parent).push_children(children);
    }
}

/// Query filter for running behaviors
#[derive(WorldQuery)]
pub struct BehaviorRunQuery {
    _node: With<BehaviorNode>,
    _cursor: With<BehaviorCursor>,
    _running: With<BehaviorRunning>,
    _paused: Without<BehaviorPaused>,
    _failure: Without<BehaviorFailure>,
    _success: Without<BehaviorSuccess>,
}

/// Query filter for idle behaviors
/// Same as running, but without the cursor
#[derive(WorldQuery)]
pub struct BehaviorIdleQuery {
    _node: With<BehaviorNode>,
    _running: With<BehaviorRunning>,
    _paused: Without<BehaviorPaused>,
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
    child_running: Option<&'static BehaviorRunning>,
}

#[derive(WorldQuery)]
pub struct BehaviorChildQueryFilter {
    _node: With<BehaviorNode>,
    _cursor: Without<BehaviorCursor>,
    _running: Without<BehaviorRunning>,
}

#[derive(Default, Debug, Clone, Deref, DerefMut, PartialEq, Resource)]
pub struct BehaviorTrace(pub Vec<String>);
impl BehaviorTrace {
    pub fn from_list(traces: &[&str]) -> Self {
        Self(traces.iter().map(|s| s.to_string()).collect())
    }
}

/// Clear BehaviorStarted every frame
fn clear_behavior_started(mut commands: Commands, started: Query<Entity, With<BehaviorStarted>>) {
    for entity in &mut started.iter() {
        commands.entity(entity).remove::<BehaviorStarted>();
    }
}

/// Process completed behaviors, pass cursor to parent
fn complete_behavior(
    mut commands: Commands,
    dones: Query<
        (
            Entity,
            Option<&BehaviorSuccess>,
            Option<&BehaviorFailure>,
            Option<&BehaviorParent>,
            Option<&BehaviorChildren>,
            &Name,
        ),
        BehaviorDoneQuery,
    >,
    parents: Query<Entity, (With<BehaviorChildren>, With<BehaviorRunning>)>,
    nodes: Query<
        (Entity, Option<&BehaviorChildren>),
        Or<(With<BehaviorCursor>, With<BehaviorRunning>)>,
    >,
    mut trace: Option<ResMut<BehaviorTrace>>,
) {
    for (entity, success, failure, parent, children, name) in &dones {
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
            entity.index().to_string(),
            state,
            name.to_string()
        );
        if let Some(trace) = trace.as_mut() {
            trace.push(format!("[{}] {} {}", entity.index(), state, name,));
        }
        commands.entity(entity).remove::<BehaviorRunning>();
        commands.entity(entity).remove::<BehaviorCursor>();

        // Stop all children recursively
        if let Some(children) = children {
            stop_children(&mut commands, children, &nodes);
        }

        // Pass cursor to parent, only if parent is running
        if let Some(parent) = parent {
            if parents.get(**parent).is_ok() {
                commands.entity(**parent).insert(BehaviorCursor::Return);
            }
        }
    }
}

/// Process ready behaviors, start them
fn start_behavior(
    mut commands: Commands,
    ready: Query<(Entity, Option<&BehaviorChildren>, &Name, &BehaviorCursor), BehaviorReadyQuery>,
    nodes: Query<
        (Entity, Option<&BehaviorChildren>),
        Or<(
            With<BehaviorCursor>,
            With<BehaviorRunning>,
            With<BehaviorSuccess>,
            With<BehaviorFailure>,
        )>,
    >,
    mut trace: Option<ResMut<BehaviorTrace>>,
) {
    for (entity, children, name, cursor) in &ready {
        // Reset all children recursively
        if let Some(children) = children {
            reset_children(&mut commands, children, &nodes);
        }
        // debug!("[{}] RESETNG {}", entity.id(), name.to_string());
        debug!(
            "[{}] STARTED {}",
            entity.index().to_string(),
            name.to_string()
        );
        if let Some(trace) = trace.as_mut() {
            trace.push(format!("[{}] STARTED {}", entity.index(), name));
        }

        commands.entity(entity).insert(BehaviorRunning);

        let starting = match cursor {
            BehaviorCursor::Delegate => true,
            BehaviorCursor::Return => false,
        };

        if starting {
            commands.entity(entity).insert(BehaviorStarted);
        }
    }
}

/// Stop all children nodes recursively, but keep their execution states
fn stop_children(
    commands: &mut Commands,
    children: &BehaviorChildren,
    nodes: &Query<
        (Entity, Option<&BehaviorChildren>),
        Or<(With<BehaviorCursor>, With<BehaviorRunning>)>,
    >,
) {
    for (entity, children) in nodes.iter_many(children.iter()) {
        commands.entity(entity).remove::<BehaviorCursor>();
        commands.entity(entity).remove::<BehaviorRunning>();
        if let Some(children) = children {
            stop_children(commands, children, nodes);
        }
    }
}

/// Reset all children nodes recursively and remove their execution states
fn reset_children(
    commands: &mut Commands,
    children: &BehaviorChildren,
    nodes: &Query<
        (Entity, Option<&BehaviorChildren>),
        Or<(
            With<BehaviorCursor>,
            With<BehaviorRunning>,
            With<BehaviorSuccess>,
            With<BehaviorFailure>,
        )>,
    >,
) {
    for (entity, children) in nodes.iter_many(children.iter()) {
        commands.entity(entity).remove::<BehaviorCursor>();
        commands.entity(entity).remove::<BehaviorRunning>();
        commands.entity(entity).remove::<BehaviorSuccess>();
        commands.entity(entity).remove::<BehaviorFailure>();
        if let Some(children) = children {
            reset_children(commands, children, nodes);
        }
    }
}
