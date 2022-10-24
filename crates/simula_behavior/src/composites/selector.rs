use crate::{
    BehaviorChildren, BehaviorCursor, BehaviorFailure, BehaviorInfo, BehaviorParent,
    BehaviorRunQuery, BehaviorRunning, BehaviorSuccess, BehaviorType,
};
use bevy::prelude::*;
use serde::{Deserialize, Serialize};

#[derive(Debug, Default, Component, Reflect, Clone, Deserialize, Serialize)]
pub struct Selector;

impl BehaviorInfo for Selector {
    const TYPE: BehaviorType = BehaviorType::Selector;
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
        (Without<BehaviorCursor>, Without<BehaviorRunning>),
    >,
) {
    for (entity, children) in &selectors {
        if children.is_empty() {
            commands.entity(entity).insert(BehaviorSuccess);
        } else {
            let mut done = true;
            for (child_entity, child_parent, failure, success) in
                nodes.iter_many(children.iter())
            {
                if let Some(child_parent) = **child_parent {
                    if entity == child_parent {
                        if failure.is_some() {
                            // Child failed, so we move to next child
                        } else if success.is_some() {
                            // Child succeeded, so we succeed
                            commands.entity(entity).insert(BehaviorSuccess);
                            break;
                        } else {
                            // Child is ready, pass on cursor and mark as running
                            done = false;
                            commands.entity(entity).remove::<BehaviorCursor>();
                            commands.entity(child_entity).insert(BehaviorCursor);
                            commands.entity(child_entity).insert(BehaviorRunning);
                            break;
                        }
                    }
                }
            }
            // If all children failed, complete with failure
            if done {
                commands.entity(entity).insert(BehaviorFailure);
            }
        }
    }
}
