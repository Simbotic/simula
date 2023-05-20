use crate::prelude::*;
use bevy::prelude::*;
use serde::{Deserialize, Serialize};

/// Delay will delay the execution of its child.
#[derive(Debug, Default, Component, Reflect, Clone, Deserialize, Serialize)]
pub struct Delay {
    #[serde(default)]
    pub duration: f64,
    #[serde(default)]
    pub start: f64,
}

impl BehaviorInfo for Delay {
    const TYPE: BehaviorType = BehaviorType::Decorator;
    const NAME: &'static str = "Delay";
    const DESC: &'static str = "Delays the execution of its child";
}

pub fn run(
    time: Res<Time>,
    mut commands: Commands,
    mut delays: Query<
        (
            Entity,
            &mut Delay,
            &mut BehaviorChildren,
            &mut BehaviorRunning,
        ),
        (With<Delay>, BehaviorRunQuery),
    >,
    nodes: Query<BehaviorChildQuery, BehaviorChildQueryFilter>,
) {
    for (entity, mut delay, children, mut running) in &mut delays {
        if children.is_empty() {
            commands.entity(entity).insert(BehaviorSuccess);
        } else {
            if children.len() > 1 {
                panic!("Decorator node has more than one child");
            }
            let child_entity = children[0]; // Safe because we checked for empty
            if !running.on_enter_handled {
                running.on_enter_handled = true;
                delay.start = time.elapsed_seconds_f64();
            }
            let current_time = time.elapsed_seconds_f64();
            if current_time - delay.start < delay.duration {
                continue; // We're still in delay, so don't do anything yet.
            }

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
                            commands.entity(child_entity).insert(BehaviorCursor);
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
}
