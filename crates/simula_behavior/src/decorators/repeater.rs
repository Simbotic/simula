use crate::{
    BehaviorChildQuery, BehaviorChildQueryFilter, BehaviorChildQueryItem, BehaviorChildren,
    BehaviorCursor, BehaviorInfo, BehaviorRunQuery, BehaviorRunning, BehaviorSuccess, BehaviorType,
};
use bevy::prelude::*;
use serde::{Deserialize, Serialize};

#[derive(Debug, Default, Component, Reflect, Clone, Deserialize, Serialize)]
pub enum Repeat {
    #[default]
    Forever,
    Times(u64),
    UntilSucceed,
}

/// Repeat a child until condition is met
#[derive(Debug, Default, Component, Reflect, Clone, Deserialize, Serialize)]
pub struct Repeater {
    pub repeat: Repeat,
    #[serde(default)]
    pub count: u64,
}

impl BehaviorInfo for Repeater {
    const TYPE: BehaviorType = BehaviorType::Decorator;
    const NAME: &'static str = "Repeater";
    const DESC: &'static str = "Repeat a child until condition is met";
}

pub fn run(
    mut commands: Commands,
    mut repeaters: Query<
        (Entity, &BehaviorChildren, &mut Repeater),
        (With<Repeater>, BehaviorRunQuery),
    >,
    nodes: Query<BehaviorChildQuery, BehaviorChildQueryFilter>,
) {
    for (entity, children, mut repeater) in &mut repeaters {
        if children.is_empty() {
            commands.entity(entity).insert(BehaviorSuccess);
        } else {
            if children.len() > 1 {
                warn!("Has more than one child, only the first will be used");
            }
            let child_entity = children[0]; // Safe because we checked for empty
            if let Ok(BehaviorChildQueryItem {
                child_entity,
                child_parent,
                child_failure,
                child_success,
            }) = nodes.get(child_entity)
            {
                if let Some(child_parent) = **child_parent {
                    if entity == child_parent {
                        // Child failed
                        if child_failure.is_some() {
                            match repeater.repeat {
                                // Forever, so we reset and repeat
                                Repeat::Forever => {
                                    commands.entity(entity).remove::<BehaviorRunning>();
                                }
                                // Times, so we check if we have reached the limit
                                Repeat::Times(times) => {
                                    // Repeat until we reach the limit
                                    if times > 0 && repeater.count < times {
                                        commands.entity(entity).remove::<BehaviorRunning>();
                                    } else {
                                        repeater.count = 0;
                                        commands.entity(entity).insert(BehaviorSuccess);
                                    }
                                }
                                // UntilSucceed, so we reset and repeat
                                Repeat::UntilSucceed => {
                                    commands.entity(entity).remove::<BehaviorRunning>();
                                }
                            }
                        }
                        // Child succeeded
                        else if child_success.is_some() {
                            match repeater.repeat {
                                // Forever, so we reset and repeat
                                Repeat::Forever => {
                                    commands.entity(entity).remove::<BehaviorRunning>();
                                }
                                // Times, so we check if we have reached the limit
                                Repeat::Times(times) => {
                                    if times > 0 && repeater.count < times {
                                        commands.entity(entity).remove::<BehaviorRunning>();
                                    } else {
                                        repeater.count = 0;
                                        commands.entity(entity).insert(BehaviorSuccess);
                                    }
                                }
                                // UntilSucceed, so we succeed
                                Repeat::UntilSucceed => {
                                    repeater.count = 0;
                                    commands.entity(entity).insert(BehaviorSuccess);
                                }
                            }
                        }
                        // Child is ready, pass on cursor
                        else {
                            repeater.count += 1;
                            debug!("[{}] RUNNING #{}", entity.id(), repeater.count,);
                            commands.entity(entity).remove::<BehaviorCursor>();
                            commands.entity(child_entity).insert(BehaviorCursor);
                        }
                    }
                }
            }
        }
    }
}
