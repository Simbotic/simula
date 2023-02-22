use bevy::prelude::*;
use serde::{Deserialize, Serialize};
use simula_behavior::prelude::*;

use crate::behaviors::movement::Movement;

#[derive(Debug, Default, Component, Reflect, Clone, Deserialize, Serialize)]
pub struct CopAlertedAction;

impl BehaviorInfo for CopAlertedAction {
    const TYPE: BehaviorType = BehaviorType::Action;
    const NAME: &'static str = "CopAlertedAction";
    const DESC: &'static str = "Handle the state of the Cop";
}

#[derive(Component)]
#[component(storage = "SparseSet")]
pub struct CopAlerted;

pub const COP_BRIBED_TICK_DURATION: u64 = 40;

pub fn run(
    mut commands: Commands,
    action_query: Query<(Entity, &CopAlertedAction, &BehaviorNode), BehaviorRunQuery>,
    mut query: Query<&mut Movement, With<CopAlerted>>,
) {
    for (action_entity, _, node) in &action_query {
        if let Some(cop_entity) = node.tree {
            if let Ok(mut cop_movement) = query.get_mut(cop_entity) {
                cop_movement.duration = 0.5;
                commands.entity(cop_entity).remove::<CopAlerted>();
                info!("[Cop {:?}] Started to Alert and moves faster", cop_entity);
            }
        }
        commands.entity(action_entity).insert(BehaviorSuccess);
    }
}
