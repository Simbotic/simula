use bevy::prelude::*;
use serde::{Deserialize, Serialize};
use simula_behavior::prelude::*;

use crate::components::robber::*;

#[derive(Debug, Default, Component, Reflect, Clone, Deserialize, Serialize)]
pub struct RobberCapturedAction;

impl BehaviorInfo for RobberCapturedAction {
    const TYPE: BehaviorType = BehaviorType::Action;
    const NAME: &'static str = "RobberCapturedAction";
    const DESC: &'static str = "Handle when the Robber is captured";
}

#[derive(Component)]
#[component(storage = "SparseSet")]
pub struct RobberCaptured {
    timer: Timer,
}

impl RobberCaptured {
    pub fn new() -> Self {
        Self {
            timer: Timer::from_seconds(6.0, TimerMode::Once),
        }
    }
}

pub fn run(
    mut commands: Commands,
    action_query: Query<(Entity, &RobberCapturedAction, &BehaviorNode), BehaviorRunQuery>,
    mut query: Query<(&mut Robber, &mut RobberCaptured)>,
    time: Res<Time>,
) {
    for (action_entity, _, node) in &action_query {
        if let Some(robber_entity) = node.tree {
            if let Ok((_robber, mut captured)) = query.get_mut(robber_entity) {
                captured.timer.tick(time.delta());
                if captured.timer.finished() {
                    commands.entity(robber_entity).remove::<RobberCaptured>();
                    info!(
                        "[Robber {:?}] Released from Capture. Started to Run",
                        robber_entity
                    );
                    commands.entity(action_entity).insert(BehaviorSuccess);
                }
            } else {
                commands.entity(action_entity).insert(BehaviorFailure);
            }
        }
    }
}
