use crate::prelude::*;
use bevy::prelude::*;
use serde::{Deserialize, Serialize};

/// All will run all of its children in parallel until all of them succeed.
#[derive(Default, Debug, Component, Reflect, Clone, Deserialize, Serialize, Inspectable)]
pub struct All;

impl BehaviorInfo for All {
    const TYPE: BehaviorType = BehaviorType::Composite;
    const NAME: &'static str = "All";
    const DESC: &'static str = "All behavior node";
}

pub fn run(
    mut commands: Commands,
    alls: Query<(Entity, &BehaviorChildren), (With<All>, BehaviorRunQuery)>,
    nodes: Query<BehaviorChildQuery, With<BehaviorNode>>,
) {
    for (entity, children) in &alls {
        if children.is_empty() {
            commands.entity(entity).insert(BehaviorSuccess);
        } else {
            let mut should_succeed = true;
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
                            // Child failed, so we fail
                            commands.entity(entity).insert(BehaviorFailure);
                            should_succeed = false;
                            break;
                        } else if child_success.is_some() {
                            // Child succeeded, so we move to next child
                        } else if child_running.is_some() {
                            // Child running, so we move to next child
                            commands.entity(entity).remove::<BehaviorCursor>();
                            should_succeed = false;
                        } else {
                            // Child is ready, pass on cursor
                            commands.entity(entity).remove::<BehaviorCursor>();
                            commands.entity(child_entity).insert(BehaviorCursor);
                            should_succeed = false;
                        }
                    } else {
                        // Child is not ours, so we fail
                        warn!("Child is not ours");
                        commands.entity(entity).insert(BehaviorFailure);
                        should_succeed = false;
                        break;
                    }
                } else {
                    // Child has no parent, so we fail
                    warn!("Child has no parent");
                    commands.entity(entity).insert(BehaviorFailure);
                    should_succeed = false;
                    break;
                }
            }
            // If all children succeed, complete with success
            if should_succeed {
                commands.entity(entity).insert(BehaviorSuccess);
            }
        }
    }
}
