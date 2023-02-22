use bevy::prelude::*;
use serde::{Deserialize, Serialize};
use simula_behavior::prelude::*;

use crate::common::Robot;

use super::rest::RobotRest;

#[derive(Debug, Default, Component, Reflect, Clone, Deserialize, Serialize)]
pub struct RobotMoveAction;

impl BehaviorInfo for RobotMoveAction {
    const TYPE: BehaviorType = BehaviorType::Action;
    const NAME: &'static str = "RobotRunAction";
    const DESC: &'static str = "Move the Robot if the Robot has energy";
}

#[derive(Component)]
#[component(storage = "SparseSet")]
pub struct RobotMove;

#[derive(Component, Reflect, Default)]
#[reflect(Component)]
pub struct Movement {
    pub points: Vec<Vec3>,
    pub duration: f32,
    pub elapsed: f32,
    pub direction: Vec3,
    pub index: usize,
}

pub fn calculate_movement(time: &Res<Time>, transform: &mut Transform, movement: &mut Movement) {
    // Update the elapsed time
    movement.elapsed += time.delta_seconds();

    // Calculate the lerp factor based on the elapsed time
    let t = movement.elapsed / movement.duration;
    let t = t.clamp(0.0, 1.0);

    // Get the start and end points for this segment of the rectangular path
    let start = movement.points[movement.index];
    let end = movement.points[(movement.index + 1) % 4];

    // Interpolate the position between the start and end points
    let position = start.lerp(end, t);

    // Update the transform component
    transform.translation = position;

    // Check if the object has reached the end of the current segment of the rectangular path
    if t >= 1.0 {
        movement.index = (movement.index + 1) % 4;
        movement.elapsed = 0.0;
        movement.direction = (movement.points[movement.index]
            - movement.points[(movement.index + 3) % 4])
            .normalize();
    }
}

pub fn run<T>(
    mut commands: Commands,
    action_query: Query<(Entity, &RobotMoveAction, &BehaviorNode), BehaviorRunQuery>,
    mut query: Query<(&mut T, &mut Movement, &mut Transform), With<RobotMove>>,
    time: Res<Time>,
) where
    T: Component + Robot,
{
    for (action_entity, _, node) in &action_query {
        if let Some(robot_entity) = node.tree {
            if let Ok((mut robot, mut movement, mut transform)) = query.get_mut(robot_entity) {
                let robot_energy = robot.get_energy();
                if robot_energy > 0 {
                    calculate_movement(&time, &mut transform, &mut movement);
                    robot.set_energy(robot_energy - 1);
                } else {
                    commands
                        .entity(robot_entity)
                        .remove::<RobotMove>()
                        .insert(RobotRest);
                    return;
                }
            }
        }
        commands.entity(action_entity).insert(BehaviorSuccess);
    }
}
