use bevy::prelude::*;
use serde::{Deserialize, Serialize};
use simula_behavior::prelude::*;

use crate::common::Robot;

#[derive(Debug, Default, Component, Reflect, Clone, Deserialize, Serialize)]
pub struct RobotRestAction;

impl BehaviorInfo for RobotRestAction {
    const TYPE: BehaviorType = BehaviorType::Action;
    const NAME: &'static str = "RobotRestAction";
    const DESC: &'static str = "Make the Robot rest if the Robot has no energy";
}

#[derive(Component)]
#[component(storage = "SparseSet")]
pub struct RobotRest;

pub fn run<T>(
    time: Res<Time>,
    mut commands: Commands,
    mut action_query: Query<
        (
            Entity,
            &RobotRestAction,
            &BehaviorNode,
            &mut BehaviorRunning,
        ),
        BehaviorRunQuery,
    >,
    mut query: Query<(&mut T, Option<&RobotRest>)>,
) where
    T: Component + Robot,
{
    for (action_entity, _, node, mut running) in &mut action_query {
        if let Some(robot_entity) = node.tree {
            if let Ok((mut robot, robot_rest)) = query.get_mut(robot_entity) {
                let robot_energy = robot.get_energy();
                let robot_rest_speed: f32 = robot.rest_speed() as f32 * 1000f32;
                if robot_energy == 0 && !running.on_enter_handled {
                    running.on_enter_handled = true;
                    commands.entity(robot_entity).insert(RobotRest);
                }
                if running.on_enter_handled {
                    if robot_energy < robot.starting_energy() {
                        robot.set_energy(
                            robot_energy
                                + ((robot_rest_speed * time.delta_seconds() * 10f32) / 1000f32)
                                    as u64,
                        );
                    } else {
                        commands.entity(robot_entity).remove::<RobotRest>();
                        commands.entity(action_entity).insert(BehaviorSuccess);
                    }
                    if robot_rest.is_none() {
                        commands.entity(action_entity).insert(BehaviorSuccess);
                    }
                } else {
                    commands.entity(action_entity).insert(BehaviorFailure);
                }
            }
        }
    }
}
