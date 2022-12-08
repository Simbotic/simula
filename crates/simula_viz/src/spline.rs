use crate::lines::*;
use bevy::prelude::*;
use simula_core::spline::*;

pub struct SplinePlugin;

impl Plugin for SplinePlugin {
    fn build(&self, app: &mut App) {
        app.add_system(spline_lines);
    }
}

#[derive(Bundle, Default)]
pub struct SplineBundle {
    pub spline: Spline,
    pub lines: Lines,
    pub mesh: Handle<Mesh>,
    pub material: Handle<LinesMaterial>,
    pub transform: Transform,
    pub global_transform: GlobalTransform,
    pub visibility: Visibility,
    pub computed_visibility: ComputedVisibility,
}

fn spline_lines(mut query: Query<(&mut Lines, &Spline, &Visibility), With<Handle<LinesMaterial>>>) {
    for (mut lines, spline, visibility) in query.iter_mut() {
        if visibility.is_visible {
            for segment in &spline.segments {
                lines.line_colored(segment.p0, segment.p1, Color::CYAN);
                lines.line_colored(segment.p3, segment.p2, Color::FUCHSIA);

                lines.cross_colored(segment.p0, 0.1, Color::WHITE);
                lines.box_colored(segment.p1, 0.1, Color::WHITE);
                lines.box_colored(segment.p2, 0.1, Color::WHITE);
                lines.cross_colored(segment.p3, 0.1, Color::WHITE);

                for i in 0..100 {
                    let t0 = (i + 0) as f32 / 100.00;
                    let t1 = (i + 1) as f32 / 100.00;
                    let p0 = segment.get_point(t0);
                    let p1 = segment.get_point(t1);
                    lines.line_colored(p0, p1, Color::WHITE);
                }
            }
        }
    }
}
