use crate::prelude::*;
use bevy::prelude::*;
use serde::{Deserialize, Serialize};
use std::borrow::Cow;
use std::fmt::Debug;

/// Subtree connects a behavior subtree to the current behavior tree.
#[derive(Debug, Component, Reflect, Clone, Default, Deserialize, Serialize)]
pub struct Subtree<T: BehaviorFactory> {
    /// Behavior asset to load.
    pub asset: Cow<'static, str>,
    /// Unload the subtree when completed.
    #[serde(default)]
    pub unload: bool,
    #[serde(skip)]
    #[reflect(ignore)]
    phantom: std::marker::PhantomData<T>,
}

impl<T: BehaviorFactory> Subtree<T> {
    pub fn type_name() -> &'static str {
        std::any::type_name::<T>()
    }
}

impl<T> BehaviorSpec for Subtree<T>
where
    T: BehaviorFactory + TypePath,
{
    // A subtree, once connected works as a decorator node
    const TYPE: BehaviorType = BehaviorType::Subtree;
    const NAME: &'static str = "Subtree";
    const ICON: &'static str = "🏃";
    const DESC: &'static str = "Connects a behavior subtree to this node";
}

impl<T> BehaviorUI for Subtree<T> where T: BehaviorFactory + TypePath {}

pub fn run<T: BehaviorFactory>(
    mut commands: Commands,
    mut subtrees: Query<
        (
            Entity,
            &BehaviorChildren,
            &Subtree<T>,
            Option<&BehaviorTree<T>>,
        ),
        BehaviorRunQuery,
    >,
    nodes: Query<BehaviorChildQuery, BehaviorChildQueryFilter>,
    asset_server: Res<AssetServer>,
) {
    for (entity, children, subtree, child_tree) in &mut subtrees {
        if child_tree.is_none() {
            let asset_path = subtree.asset.clone().into_owned();
            let behavior_document: Handle<BehaviorDocument> = asset_server.load(asset_path);
            commands
                .entity(entity)
                .insert(behavior_document)
                .insert(BehaviorTree::<T>::default())
                .insert(BehaviorTreeReset::<T>::default());
        } else if children.is_empty() {
            // Can be empty while loading subtree
        } else {
            if children.len() > 1 {
                error!("Decorator node has more than one child");
                commands.entity(entity).insert(BehaviorFailure);
                continue;
            }
            let child_entity = children[0]; // Safe because we checked for empty
            if let Ok(BehaviorChildQueryItem {
                child_entity,
                child_parent: _,
                child_failure,
                child_success,
                child_running: _,
            }) = nodes.get(child_entity)
            {
                let mut done = false;

                // Child failed, we fail
                if child_failure.is_some() {
                    commands.entity(entity).insert(BehaviorFailure);
                    done = true;
                }
                // Child succeeded, so we succeed
                else if child_success.is_some() {
                    commands.entity(entity).insert(BehaviorSuccess);
                    done = true;
                }
                // Child is ready, pass on cursor
                else {
                    commands.entity(entity).remove::<BehaviorCursor>();
                    commands
                        .entity(child_entity)
                        .insert(BehaviorCursor::Delegate);
                }

                if done && subtree.unload {
                    commands
                        .entity(entity)
                        .insert(BehaviorChildren::default())
                        .remove::<BehaviorTree<T>>();
                    commands.entity(child_entity).despawn_recursive();
                }
            }
        }
    }
}
