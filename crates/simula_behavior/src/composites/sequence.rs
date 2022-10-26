use crate::{
    BehaviorChildren, BehaviorCursor, BehaviorFailure, BehaviorInfo, BehaviorNode, BehaviorParent,
    BehaviorRunQuery, BehaviorRunning, BehaviorSuccess, BehaviorType,
};
use bevy::prelude::*;
use serde::{Deserialize, Serialize};

/// A sequence will visit each child in order, starting with the first, and when that
/// succeeds will call the second, and so on down the list of children. If any child
/// fails it will immediately return failure to the parent. If the last child in the
/// sequence succeeds, then the sequence will return success to its parent.
#[derive(Debug, Default, Component, Reflect, Clone, Deserialize, Serialize)]
pub struct Sequence;

impl BehaviorInfo for Sequence {
    const TYPE: BehaviorType = BehaviorType::Composite;
    const NAME: &'static str = "Sequence";
    const DESC: &'static str = "Sequence behavior node";
}

pub fn run(
    mut commands: Commands,
    sequences: Query<(Entity, &BehaviorChildren), (With<Sequence>, BehaviorRunQuery)>,
    nodes: Query<
        (
            Entity,
            &BehaviorParent,
            Option<&BehaviorFailure>,
            Option<&BehaviorSuccess>,
        ),
        (
            With<BehaviorNode>,
            Without<BehaviorCursor>,
            Without<BehaviorRunning>,
        ),
    >,
) {
    for (entity, children) in &sequences {
        if children.is_empty() {
            commands.entity(entity).insert(BehaviorSuccess);
        } else {
            let mut done = true;
            for (child_entity, child_parent, failure, success) in nodes.iter_many(children.iter()) {
                if let Some(child_parent) = **child_parent {
                    if entity == child_parent {
                        if failure.is_some() {
                            // Child failed, so we fail
                            commands.entity(entity).insert(BehaviorFailure);
                            break;
                        } else if success.is_some() {
                            // Child succeeded, so we move to next child
                        } else {
                            // Child is ready, pass on cursor and mark as running
                            done = false;
                            commands.entity(entity).remove::<BehaviorCursor>();
                            commands.entity(child_entity).insert(BehaviorCursor);
                            break;
                        }
                    }
                }
            }
            // If all children succeed, complete with success
            if done {
                commands.entity(entity).insert(BehaviorSuccess);
            }
        }
    }
}
