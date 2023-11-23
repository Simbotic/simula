use crate::lines::*;
use bevy::prelude::*;
use simula_core::spline::*;

pub struct SplinePlugin;

impl Plugin for SplinePlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Update, spline_lines);
    }
}

#[derive(Component)]
pub struct SplineGizmo {
    pub color: Color,
    pub p0_p1_color: Color,
    pub p3_p2_color: Color,
    pub p0_color: Color,
    pub p1_color: Color,
    pub p2_color: Color,
    pub p3_color: Color,
}

impl Default for SplineGizmo {
    fn default() -> Self {
        Self {
            color: Color::WHITE,
            p0_p1_color: Color::CYAN,
            p3_p2_color: Color::FUCHSIA,
            p0_color: Color::RED,
            p1_color: Color::GREEN,
            p2_color: Color::GREEN,
            p3_color: Color::BLUE,
        }
    }
}

#[derive(Bundle, Default)]
pub struct SplineBundle {
    pub spline: Spline,
    pub spline_gizmo: SplineGizmo,
    pub lines: Lines,
    pub transform: Transform,
    pub global_transform: GlobalTransform,
    pub visibility: Visibility,
    pub inherited_visibility: InheritedVisibility,
    pub view_visibility: ViewVisibility,
}

fn spline_lines(mut query: Query<(&mut Lines, &Spline, &SplineGizmo, &Visibility)>) {
    for (mut lines, spline, gizmo, visibility) in query.iter_mut() {
        match visibility {
            Visibility::Visible | Visibility::Inherited => {
                for segment in &spline.segments {
                    lines.line_colored(segment.p0, segment.p1, gizmo.p0_p1_color);
                    lines.line_colored(segment.p3, segment.p2, gizmo.p3_p2_color);

                    lines.cross_colored(segment.p0, 0.1, gizmo.p0_color);
                    lines.box_colored(segment.p1, 0.1, gizmo.p1_color);
                    lines.box_colored(segment.p2, 0.1, gizmo.p2_color);
                    lines.sphere_colored(segment.p3, 0.1, gizmo.p3_color);

                    for i in 0..100 {
                        let t0 = i as f32 / 100.00;
                        let t1 = (i + 1) as f32 / 100.00;
                        let p0 = segment.get_point(t0);
                        let p1 = segment.get_point(t1);
                        lines.line_colored(p0, p1, gizmo.color);
                    }
                }
            }
            Visibility::Hidden => {}
        }
    }
}
