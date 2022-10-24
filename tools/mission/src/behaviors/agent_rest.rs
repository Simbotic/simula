use bevy::prelude::*;
use simula_behavior::{BehaviorInfo, BehaviorType};

#[derive(Default, Component, Reflect, Clone)]
pub struct AgentRest;

impl BehaviorInfo for AgentRest {
    const TYPE: BehaviorType = BehaviorType::Action;
    const NAME: &'static str = "Agent rest";
    const DESC: &'static str = "Take a break for a bit";
}
