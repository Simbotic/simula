use bevy::prelude::*;
use simula_behavior::{BehaviorInfo, BehaviorRunQuery, BehaviorSuccess, BehaviorType};

#[derive(Default, Component, Reflect, Clone)]
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
