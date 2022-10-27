use crate::{
    BehaviorChildQuery, BehaviorChildQueryFilter, BehaviorChildQueryItem, BehaviorChildren,
    BehaviorCursor, BehaviorFailure, BehaviorInfo, BehaviorRunQuery, BehaviorSuccess, BehaviorType,
};
use bevy::prelude::*;
use serde::{Deserialize, Serialize};

/// A succeeder will always return success, irrespective of what the child node
/// actually returned. These are useful in cases where you want to process a branch
/// of a tree where a failure is expected or anticipated, but you don’t want to
/// abandon processing of a sequence that branch sits on.
#[derive(Debug, Default, Component, Reflect, Clone, Deserialize, Serialize)]
pub struct Succeeder;

impl BehaviorInfo for Succeeder {
    const TYPE: BehaviorType = BehaviorType::Decorator;
    const NAME: &'static str = "Succeeder";
    const DESC: &'static str = "A succeeder will always return success";
}

pub fn run(
    mut commands: Commands,
    mut succeeders: Query<(Entity, &BehaviorChildren), (With<Succeeder>, BehaviorRunQuery)>,
    nodes: Query<BehaviorChildQuery, BehaviorChildQueryFilter>,
) {
    for (entity, children) in &mut succeeders {
        if children.is_empty() {
            commands.entity(entity).insert(BehaviorSuccess);
        } else {
            if children.len() > 1 {
                warn!("Has more than one child, only the first will be used");
            }
            let child_entity = children[0]; // Safe because we checked for empty
            if let Ok(BehaviorChildQueryItem {
                child_entity,
                child_parent,
                child_failure,
                child_success,
            }) = nodes.get(child_entity)
            {
                if let Some(child_parent) = **child_parent {
                    if entity == child_parent {
                        // Child failed, so we succeed
                        if child_failure.is_some() {
                            commands.entity(entity).insert(BehaviorSuccess);
                        }
                        // Child succeeded, so we succeed
                        else if child_success.is_some() {
                            commands.entity(entity).insert(BehaviorSuccess);
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
                    }
                } else {
                    // Child has no parent, so we fail
                    warn!("Child has no parent");
                    commands.entity(entity).insert(BehaviorFailure);
                }
            }
        }
    }
}
