use crate::{
    BehaviorChildren, BehaviorCursor, BehaviorFailure, BehaviorInfo, BehaviorNode, BehaviorParent,
    BehaviorRunQuery, BehaviorRunning, BehaviorSuccess, BehaviorType,
};
use bevy::prelude::*;
use serde::{Deserialize, Serialize};

/// A selector will return a success if any of its children succeed and not process
/// any further children. It will process the first child, and if it fails will
/// process the second, until a success is reached, at which point it will instantly
/// return success. It will fail if all children fail.
#[derive(Debug, Default, Component, Reflect, Clone, Deserialize, Serialize)]
pub struct Selector;

impl BehaviorInfo for Selector {
    const TYPE: BehaviorType = BehaviorType::Composite;
    const NAME: &'static str = "Selector";
    const DESC: &'static str = "Selector behavior node";
}

pub fn run(
    mut commands: Commands,
    selectors: Query<(Entity, &BehaviorChildren), (With<Selector>, BehaviorRunQuery)>,
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
    for (entity, children) in &selectors {
        if children.is_empty() {
            commands.entity(entity).insert(BehaviorSuccess);
        } else {
            let mut should_fail = true;
            for (child_entity, child_parent, failure, success) in nodes.iter_many(children.iter()) {
                if let Some(child_parent) = **child_parent {
                    if entity == child_parent {
                        if failure.is_some() {
                            // Child failed, so we move to next child
                        } else if success.is_some() {
                            // Child succeeded, so we succeed
                            commands.entity(entity).insert(BehaviorSuccess);
                            should_fail = false;
                            break;
                        } else {
                            // Child is ready, pass on cursor
                            commands.entity(entity).remove::<BehaviorCursor>();
                            commands.entity(child_entity).insert(BehaviorCursor);
                            should_fail = false;
                            break;
                        }
                    } else {
                        // Child is not ours, so we fail
                        warn!("Child is not ours");
                        commands.entity(entity).insert(BehaviorFailure);
                        should_fail = true;
                        break;
                    }
                } else {
                    // Child has no parent, so we fail
                    warn!("Child has no parent, so we fail");
                    commands.entity(entity).insert(BehaviorFailure);
                    should_fail = true;
                    break;
                }
            }
            // If all children failed, complete with failure
            if should_fail {
                commands.entity(entity).insert(BehaviorFailure);
            }
        }
    }
}
