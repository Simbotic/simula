use crate::prelude::*;
use bevy::prelude::*;
use serde::{Deserialize, Serialize};

/// Inverts result of their child node. Success becomes failure, and failure becomes success.
#[derive(Debug, Default, Component, Reflect, Clone, Deserialize, Serialize)]
pub struct Inverter;

impl BehaviorInfo for Inverter {
    const TYPE: BehaviorType = BehaviorType::Decorator;
    const NAME: &'static str = "Inverter";
    const DESC: &'static str = "Inverts result of child node";
}

pub fn run(
    mut commands: Commands,
    mut inverters: Query<(Entity, &BehaviorChildren), (With<Inverter>, BehaviorRunQuery)>,
    nodes: Query<BehaviorChildQuery, BehaviorChildQueryFilter>,
) {
    for (entity, children) in &mut inverters {
        if children.len() != 1 {
            error!("Decorator node requires one child");
            commands.entity(entity).insert(BehaviorFailure);
            continue;
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
            if let Some(child_parent) = **child_parent {
                if entity == child_parent {
                    // Child failed, so we succeed
                    if child_failure.is_some() {
                        commands.entity(entity).insert(BehaviorSuccess);
                    }
                    // Child succeeded, so we fail
                    else if child_success.is_some() {
                        commands.entity(entity).insert(BehaviorFailure);
                    }
                    // Child is ready, pass on cursor
                    else {
                        commands.entity(entity).remove::<BehaviorCursor>();
                        commands
                            .entity(child_entity)
                            .insert(BehaviorCursor::Delegate);
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
