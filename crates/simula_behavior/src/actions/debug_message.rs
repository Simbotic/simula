use crate::{BehaviorInfo, BehaviorRunQuery, BehaviorSuccess, BehaviorType};
use bevy::prelude::*;

#[derive(Default, Component, Reflect, Clone)]
pub struct DebugMessage {
    pub message: String,
}

impl BehaviorInfo for DebugMessage {
    const TYPE: BehaviorType = BehaviorType::Action;
    const NAME: &'static str = "AgentRest";
    const DESC: &'static str = "Rest for a bit";
}

pub fn run(
    mut commands: Commands,
    debug_messages: Query<(Entity, &mut DebugMessage), BehaviorRunQuery>,
) {
    for (entity, debug_message) in &debug_messages {
        debug!("{}", debug_message.message);
        commands.entity(entity).insert(BehaviorSuccess);
    }
}
