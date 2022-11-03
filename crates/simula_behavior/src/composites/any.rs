use crate::prelude::*;
use bevy::prelude::*;
use serde::{Deserialize, Serialize};

/// Any will run all of its children in parallel until one of them succeed.
#[derive(Default, Debug, Component, Reflect, Clone, Deserialize, Serialize, Inspectable)]
pub struct Any;

impl BehaviorInfo for Any {
    const TYPE: BehaviorType = BehaviorType::Composite;
    const NAME: &'static str = "Any";
    const DESC: &'static str = "Any behavior node";
}

pub fn run(
    mut commands: Commands,
    anys: Query<(Entity, &BehaviorChildren), (With<Any>, BehaviorRunQuery)>,
    nodes: Query<BehaviorChildQuery, With<BehaviorNode>>,
) {
    for (entity, children) in &anys {
        if children.is_empty() {
            commands.entity(entity).insert(BehaviorSuccess);
        } else {
            let mut should_fail = true;
            for BehaviorChildQueryItem {
                child_entity,
                child_parent,
                child_failure,
                child_success,
                child_running,
            } in nodes.iter_many(children.iter())
            {
                if let Some(child_parent) = **child_parent {
                    if entity == child_parent {
                        if child_failure.is_some() {
                            // Child failed, so we move to next child
                        } else if child_success.is_some() {
                            // Child succeeded, so we succeed
                            commands.entity(entity).insert(BehaviorSuccess);
                            should_fail = false;
                            break;
                        } else if child_running.is_some() {
                            // Child running, so we move to next child
                            commands.entity(entity).remove::<BehaviorCursor>();
                            should_fail = false;
                        } else {
                            // Child is ready, pass on cursor
                            commands.entity(entity).remove::<BehaviorCursor>();
                            commands.entity(child_entity).insert(BehaviorCursor);
                            should_fail = false;
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
                    warn!("Child has no parent");
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
