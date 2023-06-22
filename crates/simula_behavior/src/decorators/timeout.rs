use crate::prelude::*;
use bevy::prelude::*;
use bevy_inspector_egui::prelude::*;
use serde::{Deserialize, Serialize};

#[derive(
    Debug, Default, Component, Reflect, FromReflect, Clone, Deserialize, Serialize, InspectorOptions,
)]
#[reflect(InspectorOptions)]
pub struct Timeout {
    #[serde(default)]
    #[inspector(min = 0.0, max = f64::MAX)]
    pub duration: f64,
    #[serde(skip)]
    #[reflect(ignore)]
    pub start: f64,
}

/// Timeout will fail if its child does not return within the given time limit.
impl BehaviorInfo for Timeout {
    const TYPE: BehaviorType = BehaviorType::Decorator;
    const NAME: &'static str = "Timeout";
    const DESC: &'static str = "Fails if its child does not return within the given time limit";
}

pub fn run(
    time: Res<Time>,
    mut commands: Commands,
    mut timeouts: Query<
        (
            Entity,
            &mut Timeout,
            &mut BehaviorChildren,
            Option<&BehaviorStarted>,
        ),
        (With<Timeout>, BehaviorRunQuery),
    >,
    nodes: Query<BehaviorChildQuery, BehaviorChildQueryFilter>,
) {
    for (entity, mut timeout, children, started) in &mut timeouts {
        if children.len() != 1 {
            error!("Decorator node requires one child");
            commands.entity(entity).insert(BehaviorFailure);
            continue;
        }

        let elapsed = time.elapsed_seconds_f64();
        if started.is_some() {
            timeout.start = elapsed;
        }

        if elapsed - timeout.start > timeout.duration - f64::EPSILON {
            // Time limit reached, we fail
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
