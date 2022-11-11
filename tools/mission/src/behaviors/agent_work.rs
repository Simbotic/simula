use bevy::prelude::*;
use serde::{Deserialize, Serialize};
use simula_behavior::prelude::*;
use simula_mission::{agent::Agent, machine::Machine};

#[derive(Debug, Default, Component, Reflect, Clone, Serialize, Deserialize, Inspectable)]
pub struct AgentWork {
    #[serde(default)]
    duration: f64,
    #[serde(default)]
    start: f64,
}

impl BehaviorInfo for AgentWork {
    const TYPE: BehaviorType = BehaviorType::Action;
    const NAME: &'static str = "Agent work";
    const DESC: &'static str = "Do some work";
}

pub fn run(
    mut commands: Commands,
    time: Res<Time>,
    mut agents_work: Query<
        (
            Entity,
            &mut AgentWork,
            &mut BehaviorRunning,
            &mut BehaviorNode,
        ),
        BehaviorRunQuery,
    >,
    machines: Query<(Entity, &Transform), (With<Machine>, Without<Agent>)>,
    mut agents: Query<(Entity, &mut Transform), (With<Agent>, Without<Machine>)>,
) {
    for (agent_work_entity, mut work, mut running, node) in agents_work.iter_mut() {
        if let Some(tree_entity) = node.tree {
            for (agent, mut agent_transform) in agents.iter_mut() {
                if agent == tree_entity {
                    for (_machine, machine_transform) in &machines {
                        agent_transform.translation = machine_transform.translation;
                        break;
                    }
                }
            }
        }
        if !running.on_enter_handled {
            running.on_enter_handled = true;
            work.start = time.seconds_since_startup();
        }
        let duration = time.seconds_since_startup() - work.start;
        if duration > work.duration {
            commands.entity(agent_work_entity).insert(BehaviorSuccess);
        }
    }
}
