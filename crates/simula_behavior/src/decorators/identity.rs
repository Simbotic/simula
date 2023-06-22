use crate::prelude::*;
use bevy::prelude::*;
use serde::{Deserialize, Serialize};

/// Identity decorator. It returns the same result as its child.
#[derive(Debug, Default, Component, Reflect, FromReflect, Clone, Deserialize, Serialize)]
pub struct Identity;

impl BehaviorInfo for Identity {
    const TYPE: BehaviorType = BehaviorType::Decorator;
    const NAME: &'static str = "Identity";
    const ICON: &'static str = "=";
    const DESC: &'static str = "Returns the same result as its child";
}

pub fn run(
    mut commands: Commands,
    mut identities: Query<(Entity, &BehaviorChildren), (With<Identity>, BehaviorRunQuery)>,
    nodes: Query<BehaviorChildQuery, BehaviorChildQueryFilter>,
) {
    for (entity, children) in &mut identities {
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
            // Child failed, so we fail
            if child_failure.is_some() {
                commands.entity(entity).insert(BehaviorFailure);
            }
            // Child succeeded, so we succeed
            else if child_success.is_some() {
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
