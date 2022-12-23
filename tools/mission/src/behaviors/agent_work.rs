use bevy::prelude::*;
use serde::{Deserialize, Serialize};
use simula_behavior::prelude::*;
use simula_mission::{
    agent::{Agent, AgentProductionType},
    asset_info::AssetInfo,
    machine::{Machine, MachineType},
};

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

pub struct AgentWorkNodePlugin<T: AssetInfo>(pub T);

impl<T: AssetInfo> Plugin for AgentWorkNodePlugin<T> {
    fn build(&self, app: &mut App) {
        app.add_system(run::<T>);
    }
}

pub fn run<T: AssetInfo>(
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
    machines: Query<(Entity, &Transform, &MachineType<T>), (With<Machine>, Without<Agent>)>,
    mut agents: Query<
        (Entity, &mut Transform, &AgentProductionType<T>),
        (With<Agent>, Without<Machine>),
    >,
) {
    for (agent_work_entity, mut work, mut running, node) in agents_work.iter_mut() {
        if let Some(tree_entity) = node.tree {
            if let Ok((_agent, mut agent_transform, agent_production_type)) =
                agents.get_mut(tree_entity)
            {
                for (_machine, machine_transform, machine_type) in &machines {
                    if agent_production_type.0.name() == machine_type.0.name() {
                        agent_transform.translation = machine_transform.translation;
                        break;
                    }
                }
            }
        }
        if !running.on_enter_handled {
            running.on_enter_handled = true;
            work.start = time.elapsed_seconds_f64();
        }
        let duration = time.elapsed_seconds_f64() - work.start;
        if duration > work.duration {
            commands.entity(agent_work_entity).insert(BehaviorSuccess);
        }
    }
}
