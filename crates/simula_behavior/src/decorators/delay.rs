use crate::prelude::*;
use bevy::prelude::*;
use bevy_inspector_egui::prelude::*;
use serde::{Deserialize, Serialize};

/// Delay will delay the execution of its child.
#[derive(Debug, Default, Component, Reflect, Clone, Deserialize, Serialize, InspectorOptions)]
#[reflect(InspectorOptions)]
pub struct Delay {
    #[serde(default)]
    #[inspector(min = 0.0, max = f64::MAX)]
    pub duration: f64,
    #[serde(skip)]
    pub start: f64,
    #[serde(skip)]
    pub ticks: u64,
}

impl BehaviorInfo for Delay {
    const TYPE: BehaviorType = BehaviorType::Decorator;
    const NAME: &'static str = "⌛ Delay";
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
            Option<&BehaviorStarted>,
            Option<&BehaviorCursor>,
        ),
        (With<Delay>, BehaviorRunQuery),
    >,
    nodes: Query<BehaviorChildQuery, BehaviorChildQueryFilter>,
) {
    for (entity, mut delay, children, started, cursor) in &mut delays {
        delay.ticks += 1;

        if children.len() != 1 {
            error!("Decorator node requires one child");
            commands.entity(entity).insert(BehaviorFailure);
            continue;
        }

        let elapsed = time.elapsed_seconds_f64();
        if started.is_some() {
            delay.start = elapsed;
            println!("[{}] Delay started with {:?}", entity.index(), cursor);
        }

        if elapsed - delay.start < delay.duration + f64::EPSILON {
            continue; // We're still in delay, so don't do anything yet.
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
