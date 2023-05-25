use crate::prelude::*;
use bevy::prelude::*;
use serde::{Deserialize, Serialize};

#[derive(Debug, Default, Reflect, Clone, Deserialize, Serialize)]
pub enum Repeat {
    #[default]
    Forever,
    Times(u64),
    UntilFailure,
    UntilSuccess,
}

/// Repeat a child until condition is met
#[derive(Debug, Default, Component, Reflect, Clone, Deserialize, Serialize)]
pub struct Repeater {
    pub repeat: Repeat,
    #[serde(skip)]
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
        (
            Entity,
            &BehaviorChildren,
            &mut Repeater,
            &mut BehaviorRunning,
        ),
        (With<Repeater>, BehaviorRunQuery),
    >,
    nodes: Query<BehaviorChildQuery, BehaviorChildQueryFilter>,
) {
    for (entity, children, mut repeater, mut running) in &mut repeaters {
        if children.len() != 1 {
            error!("Decorator node requires one child");
            commands.entity(entity).insert(BehaviorFailure);
            continue;
        }

        if !running.on_enter_handled {
            running.on_enter_handled = true;
            repeater.count = 0;
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
                            commands.entity(entity).insert(BehaviorSuccess);
                        }
                    }
                    // Until fail, so we succeed
                    Repeat::UntilFailure => {
                        commands.entity(entity).insert(BehaviorSuccess);
                    }
                    // Until success, so we reset and repeat
                    Repeat::UntilSuccess => {
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
                            commands.entity(entity).insert(BehaviorSuccess);
                        }
                    }
                    // Until fail, so we reset and repeat
                    Repeat::UntilFailure => {
                        commands.entity(entity).remove::<BehaviorRunning>();
                    }
                    // Until success, so we succeed
                    Repeat::UntilSuccess => {
                        commands.entity(entity).insert(BehaviorSuccess);
                    }
                }
            }
            // Child is ready, pass on cursor
            else {
                repeater.count += 1;
                commands.entity(entity).remove::<BehaviorCursor>();
                commands
                    .entity(child_entity)
                    .insert(BehaviorCursor::Delegate);
            }
        }
    }
}
