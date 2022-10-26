use crate::{
    BehaviorChildren, BehaviorCursor, BehaviorFailure, BehaviorInfo, BehaviorNode, BehaviorParent,
    BehaviorRunQuery, BehaviorRunning, BehaviorSuccess, BehaviorType,
};
use bevy::prelude::*;
use serde::{Deserialize, Serialize};

#[derive(Debug, Default, Component, Reflect, Clone, Deserialize, Serialize)]
pub enum Repeat {
    #[default]
    Forever,
    Times(u64),
    UntilSucceed,
    UntilFailure,
}

/// Repeat a child until condition is met
#[derive(Debug, Default, Component, Reflect, Clone, Deserialize, Serialize)]
pub struct Repeater {
    pub repeat: Repeat,
    pub repeated: u64,
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
    for (entity, children, mut repeater) in &mut repeaters {
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
                        // Child failed
                        if failure.is_some() {
                            match repeater.repeat {
                                // Forever, so we reset and repeat
                                Repeat::Forever => {
                                    commands.entity(entity).remove::<BehaviorRunning>();
                                }
                                // Times, so we check if we have reached the limit
                                Repeat::Times(times) => {
                                    // Repeat until we reach the limit
                                    if repeater.repeated < times {
                                        commands.entity(entity).remove::<BehaviorRunning>();
                                    } else {
                                        repeater.repeated = 0;
                                        commands.entity(entity).insert(BehaviorSuccess);
                                    }
                                }
                                // UntilSucceed, so we reset and repeat
                                Repeat::UntilSucceed => {
                                    commands.entity(entity).remove::<BehaviorRunning>();
                                }
                                // UntilFailure, so we succeed
                                Repeat::UntilFailure => {
                                    repeater.repeated = 0;
                                    commands.entity(entity).insert(BehaviorSuccess);
                                }
                            }
                        }
                        // Child succeeded
                        else if success.is_some() {
                            match repeater.repeat {
                                // Forever, so we reset and repeat
                                Repeat::Forever => {
                                    commands.entity(entity).remove::<BehaviorRunning>();
                                }
                                // Times, so we check if we have reached the limit
                                Repeat::Times(times) => {
                                    if repeater.repeated < times {
                                        commands.entity(entity).remove::<BehaviorRunning>();
                                    } else {
                                        repeater.repeated = 0;
                                        commands.entity(entity).insert(BehaviorSuccess);
                                    }
                                }
                                // UntilSucceed, so we succeed
                                Repeat::UntilSucceed => {
                                    repeater.repeated = 0;
                                    commands.entity(entity).insert(BehaviorSuccess);
                                }
                                // UntilFailure, so we reset and repeat
                                Repeat::UntilFailure => {
                                    commands.entity(entity).remove::<BehaviorRunning>();
                                }
                            }
                        }
                        // Child is ready, pass on cursor
                        else {
                            repeater.repeated += 1;
                            commands.entity(entity).remove::<BehaviorCursor>();
                            commands.entity(child_entity).insert(BehaviorCursor);
                        }
                    }
                }
            }
        }
    }
}
