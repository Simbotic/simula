use crate::prelude::*;
use bevy::prelude::*;
use serde::{Deserialize, Serialize};

/// Any will run all of its children in parallel until one of them succeed.
/// If all of them fail, the Any node will fail.
#[derive(Default, Debug, Component, Reflect, FromReflect, Clone, Deserialize, Serialize)]
pub struct Any;

impl BehaviorSpec for Any {
    const TYPE: BehaviorType = BehaviorType::Composite;
    const NAME: &'static str = "Any";
    const ICON: &'static str = "⇉";
    const DESC: &'static str = "Run all of its children in parallel until one of them succeed. \
        If all of them fail, the Any node will fail.";
}

impl BehaviorUI for Any {}

pub fn run(
    mut commands: Commands,
    anys: Query<(Entity, &BehaviorChildren), (With<Any>, BehaviorRunQuery)>,
    nodes: Query<BehaviorChildQuery, With<BehaviorNode>>,
) {
    for (entity, children) in &anys {
        if children.is_empty() {
            commands.entity(entity).insert(BehaviorSuccess);
        } else {
            // First check if any child succeeded
            let mut should_succeed = false;
            for BehaviorChildQueryItem {
                child_entity: _,
                child_parent: _,
                child_failure: _,
                child_success,
                child_running: _,
            } in nodes.iter_many(children.iter())
            {
                if child_success.is_some() {
                    // Child succeeded, so we succeed
                    should_succeed = true;
                    break;
                }
            }
            if should_succeed {
                commands.entity(entity).insert(BehaviorSuccess);
                // If we succeeded, we don't need to check any other child
                continue;
            }

            // Handle any other state of children
            let mut should_fail = true;
            for BehaviorChildQueryItem {
                child_entity,
                child_parent: _,
                child_failure,
                child_success: _,
                child_running,
            } in nodes.iter_many(children.iter())
            {
                if child_failure.is_some() {
                    // Child failed, so we move to next child
                } else if child_running.is_some() {
                    // Child running, so we move to next child
                    // TODO: Why do we remove the cursor here?
                    commands.entity(entity).remove::<BehaviorCursor>();
                    should_fail = false;
                } else {
                    // Child is ready, pass on cursor
                    commands.entity(entity).remove::<BehaviorCursor>();
                    commands
                        .entity(child_entity)
                        .insert(BehaviorCursor::Delegate);
                    should_fail = false;
                }
            }
            // If all children failed, complete with failure
            if should_fail {
                commands.entity(entity).insert(BehaviorFailure);
            }
        }
    }
}
