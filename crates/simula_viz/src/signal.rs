use crate::lines::Lines;
use bevy::prelude::*;
use simula_core::signal::*;

#[derive(Component)]
pub struct SignalGeneratorLine {
    pub points: Vec<Vec3>,
}

pub fn signal_generator_lines(
    time: Res<Time>,
    mut signals: Query<(&mut SignalGenerator, &mut SignalGeneratorLine, &mut Lines)>,
) {
    let mut hue = 0.0;
    let hue_dt = 360.0 / signals.iter().count() as f32;
    for (mut generator, mut signal_line, mut lines) in signals.iter_mut() {
        let num_points = signal_line.points.len();
        for i in 0..(num_points - 1) {
            signal_line.points[i].y = signal_line.points[i + 1].y;
        }

        let elapsed_time = std::time::Duration::from_secs_f32(time.elapsed_seconds());
        signal_line.points[num_points - 1].y = generator.sample(elapsed_time);

        let color = Color::Hsla {
            hue,
            lightness: 0.5,
            saturation: 1.0,
            alpha: 1.0,
        };

        for i in 0..(num_points - 1) {
            let start = signal_line.points[i];
            let end = signal_line.points[i + 1];
            lines.line_colored(start, end, color);
        }

        hue += hue_dt;
    }
}

#[derive(Component)]
pub struct SignalControlLine {
    pub points: Vec<Vec3>,
}

pub fn signal_control_lines(
    time: Res<Time>,
    mut signals: Query<(
        &mut SignalController<f32>,
        &SignalGeneratorLine,
        &mut SignalControlLine,
        &mut Lines,
    )>,
) {
    let mut hue = 100.0;
    let hue_dt = 360.0 / signals.iter().count() as f32;
    for (mut controller, signal_line, mut control_line, mut lines) in signals.iter_mut() {
        let num_points = control_line.points.len();
        for i in 0..(num_points - 1) {
            control_line.points[i].y = control_line.points[i + 1].y;
        }

        let control = controller.control(
            signal_line.points[num_points - 1].y,
            control_line.points[num_points - 1].y,
            time.delta(),
        );
        control_line.points[num_points - 1].y += control;

        let color = Color::Hsla {
            hue,
            lightness: 0.5,
            saturation: 1.0,
            alpha: 1.0,
        };

        for i in 0..(num_points - 1) {
            let start = control_line.points[i];
            let end = control_line.points[i + 1];
            lines.line_colored(start, end, color);
        }

        hue += hue_dt;
    }
}
