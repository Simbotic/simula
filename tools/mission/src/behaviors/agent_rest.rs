use bevy::prelude::*;
use serde::{Deserialize, Serialize};
use simula_behavior::prelude::*;

#[derive(Debug, Default, Component, Reflect, Clone, Serialize, Deserialize, Inspectable)]
pub struct AgentRest;

impl BehaviorInfo for AgentRest {
    const TYPE: BehaviorType = BehaviorType::Action;
    const NAME: &'static str = "Agent rest";
    const DESC: &'static str = "Take a break for a bit";
}

pub fn run(mut commands: Commands, agents: Query<(Entity, &mut AgentRest), BehaviorRunQuery>) {
    for (agent, _rest) in &agents {
        commands.entity(agent).insert(BehaviorSuccess);
    }
}
