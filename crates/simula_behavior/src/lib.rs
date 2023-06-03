use actions::*;
use asset::{Behavior, BehaviorAsset, BehaviorAssetLoader, BehaviorAssetLoading};
use bevy::{ecs::query::WorldQuery, ecs::system::EntityCommands, prelude::*, reflect::TypeUuid};
use composites::*;
use decorators::*;
use strum::AsRefStr;

pub mod actions;
pub mod asset;
pub mod color_hex_utils;
pub mod composites;
pub mod decorators;
pub mod inspector;
pub mod protocol;
pub mod test;

pub mod prelude {
    pub use crate::actions::*;
    pub use crate::asset::{
        behavior_loader, Behavior, BehaviorAsset, BehaviorAssetLoader, BehaviorAssetLoading,
    };
    pub use crate::composites::*;
    pub use crate::decorators::*;
    pub use crate::inspector::BehaviorInspectorPlugin;
    pub use crate::protocol::{self};
    pub use crate::{
        BehaviorChildQuery, BehaviorChildQueryFilter, BehaviorChildQueryItem, BehaviorChildren,
        BehaviorCursor, BehaviorFactory, BehaviorFailure, BehaviorInfo, BehaviorNode,
        BehaviorParent, BehaviorPlugin, BehaviorRunQuery, BehaviorRunning, BehaviorSuccess,
        BehaviorTree, BehaviorType,
    };
}

pub struct BehaviorPlugin;

#[derive(Bundle)]
pub struct BehaviorBundle<T>
where
    T: Reflect + Component + Clone,
{
    pub behavior: T,
    pub desc: BehaviorDesc,
    pub node: BehaviorNode,
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
            desc: BehaviorDesc {
                typ: T::TYPE,
                name: T::NAME.to_string(),
                desc: T::DESC.to_string(),
            },
            node: BehaviorNode::default(),
            name: Name::new(format!("Behavior: {}", T::NAME)),
            parent: BehaviorParent::default(),
            children: BehaviorChildren::default(),
        }
    }
}

/// How to spawn a behavior node
pub trait BehaviorFactory:
    Clone + Default + TypeUuid + Send + Sync + 'static + Default + std::fmt::Debug
{
    fn insert(&self, commands: &mut EntityCommands);

    fn label(&self) -> &str {
        std::any::type_name::<Self>()
    }

    fn typ(&self) -> BehaviorType {
        panic!("BehaviorFactory::typ() not implemented")
    }

    fn color(&self) -> Color {
        panic!("BehaviorFactory::color() not implemented")
    }

    fn reflect(&self) -> &dyn Reflect {
        panic!("BehaviorFactory::data() not implemented")
    }

    fn reflect_mut(&mut self) -> &mut dyn Reflect {
        panic!("BehaviorFactory::data() not implemented")
    }

    fn categories(&self) -> Vec<&'static str> {
        panic!("BehaviorFactory::categories() not implemented")
    }

    fn list() -> Vec<Self> {
        panic!("BehaviorFactory::list() not implemented")
    }
}

impl Plugin for BehaviorPlugin {
    fn build(&self, app: &mut App) {
        app.register_type::<BehaviorTree>()
            .register_type::<BehaviorDesc>()
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
            .add_asset::<BehaviorAsset>()
            .init_asset_loader::<BehaviorAssetLoader>()
            .add_systems(
                (complete_behavior, start_behavior)
                    .chain()
                    .in_base_set(CoreSet::PostUpdate),
            )
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
pub struct BehaviorRunning {
    pub on_enter_handled: bool,
}

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
#[derive(Debug, Default, Reflect, Clone, Component)]
#[reflect(Component)]
pub struct BehaviorNode {
    pub tree: Option<Entity>,
}

/// A marker added to behavior node entities
#[derive(Debug, Default, Reflect, Clone, Component)]
#[reflect(Component)]
pub struct BehaviorDesc {
    pub typ: BehaviorType,
    pub name: String,
    pub desc: String,
}

/// A component to point to the parent of a behavior node
#[derive(Deref, Debug, Default, Reflect, Clone, Component)]
#[reflect(Component)]
pub struct BehaviorParent(Option<Entity>);

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
}

/// A component to provide static behavior node info
pub trait BehaviorInfo
where
    Self: Reflect + Component + Clone + Default + Sized + 'static,
{
    const TYPE: BehaviorType;
    const NAME: &'static str;
    const DESC: &'static str;

    fn insert(commands: &mut EntityCommands) {
        commands.insert(BehaviorBundle::<Self>::default());
    }

    fn insert_with(commands: &mut EntityCommands, data: &Self) {
        commands.insert(BehaviorBundle::<Self> {
            behavior: data.clone(),
            ..Default::default()
        });
    }
}

pub fn add_children(commands: &mut Commands, parent: Entity, children: &[Entity]) {
    commands
        .entity(parent)
        .insert(BehaviorChildren(children.to_vec()));
    for child in children {
        commands.entity(*child).insert(BehaviorParent(Some(parent)));
    }
    commands.entity(parent).push_children(children);
}

/// A component added to identify the root of a behavior tree
#[derive(Default, Reflect, Clone, Component)]
#[reflect(Component)]
pub struct BehaviorTree {
    pub root: Option<Entity>,
}

impl BehaviorTree {
    /// Spawn behavior tree from asset.
    /// A parent is optional, but if it is provided, it must be a behavior node.
    pub fn from_asset<T>(
        tree: Entity,
        parent: Option<Entity>,
        commands: &mut Commands,
        node: Handle<BehaviorAsset>,
    ) -> Self
    where
        T: TypeUuid + Send + Sync + 'static + Default + std::fmt::Debug,
    {
        let entity = commands
            .spawn_empty()
            .insert(BehaviorAssetLoading::<T> {
                node,
                tree,
                parent,
                phantom: std::marker::PhantomData,
            })
            .id();
        Self { root: Some(entity) }
    }

    /// Spawn a behavior tree from a behavior node.
    /// A parent is optional, but if it is provided, it must be a behavior node.
    pub(crate) fn insert_tree<T>(
        tree: Entity,
        entity: Entity,
        parent: Option<Entity>,
        commands: &mut Commands,
        node: &Behavior<T>,
    ) -> Entity
    where
        T: Default + BehaviorFactory,
    {
        let Behavior(name, node_type, nodes) = node;
        let mut entity_commands = commands.entity(entity);
        node_type.insert(&mut entity_commands);
        entity_commands.insert(Name::new(name.clone()));
        entity_commands.insert(BehaviorParent(parent));
        entity_commands.insert(BehaviorNode { tree: Some(tree) });

        let children = nodes
            .iter()
            .map(|node| {
                Self::insert_tree(
                    tree,
                    commands.spawn_empty().id(),
                    Some(entity),
                    commands,
                    node,
                )
            })
            .collect::<Vec<Entity>>();
        add_children(commands, entity, &children);
        entity
    }
}

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

/// Process completed behaviors, pass cursor to parent
fn complete_behavior(
    mut commands: Commands,
    dones: Query<
        (
            Entity,
            Option<&BehaviorSuccess>,
            Option<&BehaviorFailure>,
            &BehaviorParent,
            &BehaviorChildren,
            &Name,
        ),
        BehaviorDoneQuery,
    >,
    parents: Query<Entity, (With<BehaviorParent>, With<BehaviorRunning>)>,
    nodes: Query<
        (Entity, &BehaviorChildren),
        Or<(
            With<BehaviorCursor>,
            With<BehaviorRunning>,
            With<BehaviorSuccess>,
            With<BehaviorFailure>,
        )>,
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

        // Reset all children recursively
        reset_children(true, &mut commands, children, &nodes);

        // Pass cursor to parent, only if parent is running
        if let Some(parent) = **parent {
            if parents.get(parent).is_ok() {
                commands.entity(parent).insert(BehaviorCursor::Return);
            }
        }
    }
}

/// Process ready behaviors, start them
fn start_behavior(
    mut commands: Commands,
    ready: Query<(Entity, &BehaviorChildren, &Name, &BehaviorCursor), BehaviorReadyQuery>,
    nodes: Query<
        (Entity, &BehaviorChildren),
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
        // Reset children
        reset_children(false, &mut commands, children, &nodes);
        // debug!("[{}] RESETNG {}", entity.id(), name.to_string());
        debug!(
            "[{}] STARTED {}",
            entity.index().to_string(),
            name.to_string()
        );
        if let Some(trace) = trace.as_mut() {
            trace.push(format!("[{}] STARTED {}", entity.index(), name));
        }
        let on_enter_handled = match cursor {
            BehaviorCursor::Delegate => false,
            BehaviorCursor::Return => true,
        };
        commands
            .entity(entity)
            .insert(BehaviorRunning { on_enter_handled });
    }
}

fn reset_children(
    recursively: bool,
    commands: &mut Commands,
    children: &BehaviorChildren,
    nodes: &Query<
        (Entity, &BehaviorChildren),
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
        if recursively {
            reset_children(true, commands, children, nodes);
        }
    }
}
