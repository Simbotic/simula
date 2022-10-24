use bevy::{prelude::*, reflect::TypeUuid};
use serde::{Deserialize, Serialize};
use simula_behavior::{actions::*, composites::*};

pub mod agent_rest;
pub mod agent_work;

pub struct AgentBehaviorPlugin;

impl Plugin for AgentBehaviorPlugin {
    fn build(&self, app: &mut App) {
        app.add_system(agent_rest::run).add_system(agent_work::run);
    }
}

#[derive(Serialize, Deserialize, TypeUuid)]
#[uuid = "28111cc6-5360-11ed-a639-02a179e5df2b"]
pub enum DebugNode {
    DebugAction(DebugAction),
    Selector(Selector),
    Sequence(Sequence),
}

impl Default for DebugNode {
    fn default() -> Self {
        Self::DebugAction(DebugAction::default())
    }
}

pub fn create(_commands: &mut Commands) {}
