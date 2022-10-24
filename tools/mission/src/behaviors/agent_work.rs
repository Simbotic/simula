use bevy::prelude::*;
use simula_behavior::{BehaviorInfo, BehaviorType};

#[derive(Default, Component, Reflect, Clone)]
pub struct AgentWork;

impl BehaviorInfo for AgentWork {
    const TYPE: BehaviorType = BehaviorType::Action;
    const NAME: &'static str = "Agent work";
    const DESC: &'static str = "Do some work";
}
