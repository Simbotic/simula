use bevy::prelude::*;
use serde::{Deserialize, Serialize};
use simula_behavior::prelude::*;

use crate::common::Robot;

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

pub fn calculate_movement(
    time: Res<Time>,
    mut query: Query<(&mut Transform, &mut Movement), With<RobotMove>>,
) {
    for (mut transform, mut movement) in query.iter_mut() {
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
}

pub fn run<T>(
    mut commands: Commands,
    mut action_query: Query<
        (
            Entity,
            &RobotMoveAction,
            &BehaviorNode,
            &mut BehaviorRunning,
        ),
        BehaviorRunQuery,
    >,
    mut query: Query<(&mut T, Option<&RobotMove>)>,
) where
    T: Component + Robot,
{
    for (action_entity, _, node, mut running) in &mut action_query {
        if let Some(robot_entity) = node.tree {
            if let Ok((mut robot, robot_move)) = query.get_mut(robot_entity) {
                let robot_energy = robot.get_energy();
                if running.on_enter_handled && robot_move.is_none() {
                    commands.entity(action_entity).insert(BehaviorSuccess);
                }
                if robot_energy > 0.0 {
                    if !running.on_enter_handled {
                        running.on_enter_handled = true;
                        commands.entity(robot_entity).insert(RobotMove);
                    }
                    robot.set_energy(robot_energy - 1.0);
                } else {
                    commands.entity(robot_entity).remove::<RobotMove>();
                    commands.entity(action_entity).insert(BehaviorSuccess);
                }
            }
        }
    }
}