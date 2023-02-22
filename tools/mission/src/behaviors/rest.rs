use bevy::prelude::*;
use serde::{Deserialize, Serialize};
use simula_behavior::prelude::*;

use crate::common::Robot;

use super::movement::RobotMove;

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
    mut commands: Commands,
    action_query: Query<(Entity, &RobotRestAction, &BehaviorNode), BehaviorRunQuery>,
    mut query: Query<&mut T, With<RobotRest>>,
) where
    T: Component + Robot,
{
    for (action_entity, _, node) in &action_query {
        if let Some(robot_entity) = node.tree {
            if let Ok(mut robot) = query.get_mut(robot_entity) {
                let robot_energy = robot.get_energy();
                let robot_rest_speed = robot.rest_speed();
                if robot_energy < robot.starting_energy() {
                    robot.set_energy(robot_energy + robot_rest_speed);
                } else {
                    commands
                        .entity(robot_entity)
                        .remove::<RobotRest>()
                        .insert(RobotMove);
                    return;
                }
            }
        }
        commands.entity(action_entity).insert(BehaviorSuccess);
    }
}
