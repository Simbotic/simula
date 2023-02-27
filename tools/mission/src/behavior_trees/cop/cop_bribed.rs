use bevy::prelude::*;
use serde::{Deserialize, Serialize};
use simula_behavior::prelude::*;

use crate::components::cop::Cop;

#[derive(Debug, Default, Component, Reflect, Clone, Deserialize, Serialize)]
pub struct CopBribedAction;

impl BehaviorInfo for CopBribedAction {
    const TYPE: BehaviorType = BehaviorType::Action;
    const NAME: &'static str = "CopBribedAction";
    const DESC: &'static str = "Handle the state of the Cop";
}

#[derive(Component, Default)]
#[component(storage = "SparseSet")]
pub struct CopBribed {
    timer: Timer,
}

impl CopBribed {
    pub fn new() -> Self {
        Self {
            timer: Timer::from_seconds(3.0, TimerMode::Once),
        }
    }
}

pub fn run(
    mut commands: Commands,
    action_query: Query<(Entity, &CopBribedAction, &BehaviorNode), BehaviorRunQuery>,
    mut query: Query<(&Cop, &mut CopBribed)>,
    time: Res<Time>,
) {
    for (action_entity, _, node) in &action_query {
        if let Some(cop_entity) = node.tree {
            if let Ok((_cop, mut bribed)) = query.get_mut(cop_entity) {
                bribed.timer.tick(time.delta());
                if bribed.timer.finished() {
                    commands.entity(cop_entity).remove::<CopBribed>();
                    info!(
                        "[Cop {:?}] Stopped admiring the money. Started to Chase",
                        cop_entity
                    );
                    commands.entity(action_entity).insert(BehaviorSuccess);
                }
            } else {
                commands.entity(action_entity).insert(BehaviorFailure);
            }
        }
    }
}
