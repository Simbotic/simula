use super::line;
use bevy::prelude::*;
pub struct Axes {
    pub size: f32,
}

impl Default for Axes {
    fn default() -> Self {
        Axes { size: 1. }
    }
}

#[derive(Bundle)]
pub struct AxesBundle {
    pub axes: Axes,
    pub transform: Transform,
    pub global_transform: GlobalTransform,
}

impl Default for AxesBundle {
    fn default() -> Self {
        AxesBundle {
            axes: Default::default(),
            transform: Default::default(),
            global_transform: Default::default(),
        }
    }
}

pub fn system(mut lines: ResMut<line::Lines>, query: Query<(&Axes, &Transform)>) {
    for (axes, transform) in query.iter() {
        let start = transform.translation;
        let f_end = start + transform.forward() * axes.size;
        let r_end = start + transform.right() * axes.size;
        let u_end = start + transform.up() * axes.size;
        lines.line_colored(start, f_end, 0., Color::BLUE);
        lines.line_colored(start, r_end, 0., Color::RED);
        lines.line_colored(start, u_end, 0., Color::GREEN);
    }
}
