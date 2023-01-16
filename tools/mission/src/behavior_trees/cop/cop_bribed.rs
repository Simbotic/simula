use bevy::prelude::*;
use serde::{Deserialize, Serialize};
use simula_behavior::prelude::*;

use crate::{
    common::CanRotate,
    cop::{CopBribed, CopChase},
};

#[derive(Debug, Default, Component, Reflect, Clone, Deserialize, Serialize, Inspectable)]
pub struct CopBribedAction;

impl BehaviorInfo for CopBribedAction {
    const TYPE: BehaviorType = BehaviorType::Action;
    const NAME: &'static str = "CopBribedAction";
    const DESC: &'static str = "Handle the state of the Cop";
}

pub const COP_BRIBED_TICK_DURATION: u64 = 40;

pub fn run(
    mut commands: Commands,
    action_query: Query<(Entity, &CopBribedAction, &BehaviorNode), BehaviorRunQuery>,
    query: Query<Entity, With<CopBribed>>,
    mut status_duration: Local<u64>,
) {
    for (action_entity, _, node) in &action_query {
        if let Some(cop_entity) = node.tree {
            if query.get(cop_entity).is_ok() {
                *status_duration += 1;
                if *status_duration > COP_BRIBED_TICK_DURATION {
                    *status_duration = 0;
                    commands
                        .entity(cop_entity)
                        .remove::<CopBribed>()
                        .insert(CanRotate)
                        .insert(CopChase);
                    info!(
                        "[Cop {:?}] Stopped admiring the money. Started to Chase",
                        cop_entity
                    );
                }
            }
        }
        commands.entity(action_entity).insert(BehaviorSuccess);
    }
}
