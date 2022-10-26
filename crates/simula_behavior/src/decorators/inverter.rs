use crate::{
    BehaviorChildren, BehaviorCursor, BehaviorFailure, BehaviorInfo, BehaviorNode, BehaviorParent,
    BehaviorRunQuery, BehaviorRunning, BehaviorSuccess, BehaviorType,
};
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
    nodes: Query<
        (
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
    for (entity, children) in &mut inverters {
        if children.is_empty() {
            commands.entity(entity).insert(BehaviorSuccess);
        } else {
            if children.len() > 1 {
                warn!("Has more than one child, only the first will be used");
            }
            let child_entity = children[0]; // Safe because we checked for empty
            if let Ok((parent, failure, success)) = nodes.get(child_entity) {
                if let Some(parent) = **parent {
                    if entity == parent {
                        // Child failed, so we succeed
                        if failure.is_some() {
                            commands.entity(entity).insert(BehaviorSuccess);
                        }
                        // Child succeeded, so we fail
                        else if success.is_some() {
                            commands.entity(entity).insert(BehaviorFailure);
                        }
                        // Child is ready, pass on cursor
                        else {
                            commands.entity(entity).remove::<BehaviorCursor>();
                            commands.entity(child_entity).insert(BehaviorCursor);
                        }
                    }
                }
            }
        }
    }
}
