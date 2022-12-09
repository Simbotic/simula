use crate::lines::Lines;
use bevy::prelude::*;
use simula_core::ease::*;

#[derive(Component)]
pub struct EaseLine {
    pub points: Vec<Vec3>,
    pub ease_func: EaseFunction,
}

pub fn ease_lines(time: Res<Time>, mut easings: Query<(&mut EaseLine, &mut Lines)>) {
    let color = Color::SALMON;

    for (mut ease_line, mut lines) in easings.iter_mut() {
        // Draw the bottom and top lines
        lines.line_colored(Vec3::ZERO, Vec3::new(1.0, 0.0, 0.0), Color::BLUE);
        lines.line_colored(
            Vec3::new(0.0, 1.0, 0.0),
            Vec3::new(1.0, 1.0, 0.0),
            Color::RED,
        );

        // Compute the points of the ease curve
        let num_points = ease_line.points.len();
        for i in 1..num_points {
            let t = (i as f32) / ((num_points - 1) as f32);
            ease_line.points[i].y = t.calc(ease_line.ease_func);
        }

        // Draw the animated ease line
        let t = (time.elapsed_seconds() % 1.0) as f32;
        let t = t.calc(ease_line.ease_func);
        lines.line_colored(Vec3::new(0.0, t, 0.0), Vec3::new(1.0, t, 0.0), Color::GREEN);

        // Draw the ease line
        for i in 0..(num_points - 1) {
            let start = ease_line.points[i];
            let end = ease_line.points[i + 1];
            lines.line_colored(start, end, color);
        }
    }
}
