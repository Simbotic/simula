use bevy::prelude::*;

use crate::behaviors::movement::Movement;

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
