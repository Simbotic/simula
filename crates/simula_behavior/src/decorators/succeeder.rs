use crate::prelude::*;
use bevy::prelude::*;
use serde::{Deserialize, Serialize};

/// A succeeder will always return success, irrespective of what the child node
/// actually returned. These are useful in cases where you want to process a branch
/// of a tree where a failure is expected or anticipated, but you don’t want to
/// abandon processing of a sequence that branch sits on.
#[derive(Debug, Default, Component, Reflect, FromReflect, Clone, Deserialize, Serialize)]
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
        if children.len() != 1 {
            error!("Decorator node requires one child");
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
            // Child succeeded or failed, so we succeed
            if child_success.is_some() || child_failure.is_some() {
                commands.entity(entity).insert(BehaviorSuccess);
            }
            // Child is ready, pass on cursor
            else {
                commands.entity(entity).remove::<BehaviorCursor>();
                commands
                    .entity(child_entity)
                    .insert(BehaviorCursor::Delegate);
            }
        }
    }
}
