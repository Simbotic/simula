use actions::*;
use asset::BTNode;
use bevy::{ecs::query::WorldQuery, ecs::system::EntityCommands, prelude::*};
use bevy_inspector_egui::Inspectable;
use composites::*;

pub use crate::asset::BehaviorDocument;

pub mod actions;
pub mod asset;
pub mod composites;
pub mod decorators;
pub mod editor;

pub mod my_behavior;

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
trait BehaviorSpawner {
    fn spawn_with(&self, commands: &mut EntityCommands);
}

impl Plugin for BehaviorPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugin(my_behavior::MyBehaviorPlugin)
            .register_type::<editor::BehaviorGraphState>()
            .register_type::<editor::BehaviorEditorState>()
            .register_type::<BehaviorSuccess>()
            .register_type::<BehaviorRunning>()
            .register_type::<BehaviorFailure>()
            .add_system(editor::egui_update)
            .add_system(complete_behavior)
            .add_system(sequence::run)
            .add_system(selector::run)
            .add_system(debug_action::run);
    }
}

fn gen_tree<T>(parent: Option<Entity>, commands: &mut Commands, node: &BTNode<T>) -> Entity
where
    T: Default + BehaviorSpawner,
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
