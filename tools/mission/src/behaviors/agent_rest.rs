use bevy::prelude::*;
use serde::{Deserialize, Serialize};
use simula_behavior::prelude::*;
use simula_mission::agent::Agent;

#[derive(Debug, Default, Component, Reflect, Clone, Serialize, Deserialize, Inspectable)]
pub struct AgentRest {
    #[serde(default)]
    duration: f64,
    #[serde(default)]
    start: f64,
    #[serde(default)]
    position: Vec3,
}

impl BehaviorInfo for AgentRest {
    const TYPE: BehaviorType = BehaviorType::Action;
    const NAME: &'static str = "Agent rest";
    const DESC: &'static str = "Take a break for a bit";
}

pub fn run(
    mut commands: Commands,
    time: Res<Time>,
    mut agents_rest: Query<
        (
            Entity,
            &mut AgentRest,
            &mut BehaviorRunning,
            &mut BehaviorNode,
        ),
        BehaviorRunQuery,
    >,
    mut agents: Query<(Entity, &mut Transform), With<Agent>>,
) {
    for (agent_rest_entity, mut rest, mut running, node) in agents_rest.iter_mut() {
        if let Some(tree_entity) = node.tree {
            for (agent, mut agent_transform) in agents.iter_mut() {
                if agent == tree_entity {
                    agent_transform.translation = rest.position;
                }
            }
        }

        if !running.on_enter_handled {
            running.on_enter_handled = true;
            rest.start = time.elapsed_seconds_f64();
        }
        let duration = time.elapsed_seconds_f64() - rest.start;
        if duration > rest.duration {
            commands.entity(agent_rest_entity).insert(BehaviorSuccess);
        }
    }
}
