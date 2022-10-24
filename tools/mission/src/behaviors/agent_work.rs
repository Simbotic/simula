use bevy::prelude::*;
use simula_behavior::{BehaviorInfo, BehaviorRunQuery, BehaviorSuccess, BehaviorType};

#[derive(Default, Component, Reflect, Clone)]
pub struct AgentWork;

impl BehaviorInfo for AgentWork {
    const TYPE: BehaviorType = BehaviorType::Action;
    const NAME: &'static str = "Agent work";
    const DESC: &'static str = "Do some work";
}

pub fn run(mut commands: Commands, agents: Query<(Entity, &mut AgentWork), BehaviorRunQuery>) {
    for (agent, _work) in &agents {
        commands.entity(agent).insert(BehaviorSuccess);
    }
}
