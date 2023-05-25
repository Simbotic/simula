use crate::prelude::*;
use bevy::prelude::*;
use serde::{Deserialize, Serialize};

/// All will run all of its children in parallel until all of them succeed.
/// If any of them fail, the All node will fail.
#[derive(Default, Debug, Component, Reflect, Clone, Deserialize, Serialize)]
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
            // First check if any child failed
            let mut should_fail = false;
            for BehaviorChildQueryItem {
                child_entity: _,
                child_parent,
                child_failure,
                child_success: _,
                child_running: _,
            } in nodes.iter_many(children.iter())
            {
                if let Some(child_parent) = **child_parent {
                    if entity == child_parent && child_failure.is_some() {
                        // Child failed, so we fail
                        should_fail = true;
                        break;
                    }
                }
            }
            if should_fail {
                commands.entity(entity).insert(BehaviorFailure);
                // If we failed, we don't need to check any other child
                continue;
            }

            // Handle any other state of children
            let mut should_succeed = true;
            for BehaviorChildQueryItem {
                child_entity,
                child_parent: _,
                child_failure: _,
                child_success,
                child_running,
            } in nodes.iter_many(children.iter())
            {
                if child_success.is_some() {
                    // Child succeeded, so we move to next child
                } else if child_running.is_some() {
                    // Child running, so we move to next child
                    commands.entity(entity).remove::<BehaviorCursor>();
                    should_succeed = false;
                } else {
                    // Child is ready, pass on cursor
                    commands.entity(entity).remove::<BehaviorCursor>();
                    commands
                        .entity(child_entity)
                        .insert(BehaviorCursor::Delegate);
                    should_succeed = false;
                }
            }
            // If all children succeed, complete with success
            if should_succeed {
                commands.entity(entity).insert(BehaviorSuccess);
            }
        }
    }
}
