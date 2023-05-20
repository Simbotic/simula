use crate::{add_children, prelude::*};
use bevy::prelude::*;
use serde::{Deserialize, Serialize};
use std::fmt::Debug;

/// Subtree connects a behavior subtree to the current behavior tree.
#[derive(Debug, Default, Component, Reflect, Clone, Deserialize, Serialize)]
pub struct Subtree<T: BehaviorSpawner> {
    /// Behavior asset to load.
    pub asset: String,
    /// Unload the subtree when completed.
    #[serde(default)]
    pub unload: bool,
    #[serde(skip)]
    #[reflect(ignore)]
    phantom: std::marker::PhantomData<T>,
}

impl<T> BehaviorInfo for Subtree<T>
where
    T: BehaviorSpawner,
{
    const TYPE: BehaviorType = BehaviorType::Decorator;
    const NAME: &'static str = "Subtree";
    const DESC: &'static str = "Connects a behavior subtree to this node";
}

pub fn run<T: BehaviorSpawner>(
    mut commands: Commands,
    mut subtrees: Query<
        (
            Entity,
            &BehaviorChildren,
            &Subtree<T>,
            Option<&BehaviorTree>,
        ),
        BehaviorRunQuery,
    >,
    nodes: Query<BehaviorChildQuery, BehaviorChildQueryFilter>,
    asset_server: Res<AssetServer>,
) {
    for (entity, children, subtree, child_tree) in &mut subtrees {
        if child_tree.is_none() {
            let document: Handle<BehaviorAsset> = asset_server.load(&subtree.asset);
            let behavior = BehaviorTree::from_asset::<T>(Some(entity), &mut commands, document);
            if let Some(root) = behavior.root {
                add_children(&mut commands, entity, &[root]);
            }
            commands.entity(entity).insert(behavior);
        } else if children.is_empty() {
            // Can be empty while loading subtree
        } else {
            if children.len() > 1 {
                panic!("Decorator node has more than one child");
            }
            let child_entity = children[0]; // Safe because we checked for empty
            if let Ok(BehaviorChildQueryItem {
                child_entity,
                child_parent,
                child_failure,
                child_success,
                child_running: _,
            }) = nodes.get(child_entity)
            {
                let mut done = false;
                if let Some(child_parent) = **child_parent {
                    if entity == child_parent {
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
                            commands.entity(child_entity).insert(BehaviorCursor);
                        }
                    } else {
                        // Child is not ours, so we fail
                        warn!("Child is not ours");
                        commands.entity(entity).insert(BehaviorFailure);
                        done = true;
                    }
                } else {
                    // Child has no parent, so we fail
                    warn!("Child has no parent");
                    commands.entity(entity).insert(BehaviorFailure);
                    done = true;
                }
                if done && subtree.unload {
                    commands
                        .entity(entity)
                        .insert(BehaviorChildren::default())
                        .remove::<BehaviorTree>();
                    commands.entity(child_entity).despawn_recursive();
                }
            }
        }
    }
}
