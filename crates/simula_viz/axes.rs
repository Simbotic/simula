use super::line;
use bevy::prelude::*;
pub struct Axes {
    pub size: f32,
    pub inner_offset: f32,
}

impl Default for Axes {
    fn default() -> Self {
        Axes {
            size: 1.,
            inner_offset: 0.,
        }
    }
}

#[derive(Bundle)]
pub struct AxesBundle {
    pub axes: Axes,
    pub visible: Visible,
    pub transform: Transform,
    pub global_transform: GlobalTransform,
}

impl Default for AxesBundle {
    fn default() -> Self {
        AxesBundle {
            axes: Default::default(),
            visible: Visible { is_visible: true, is_transparent: false },
            transform: Default::default(),
            global_transform: Default::default(),
        }
    }
}

pub fn system(mut lines: ResMut<line::Lines>, query: Query<(&Axes, &GlobalTransform, &Visible)>) {
    for (axes, transform, visible) in query.iter() {
        if visible.is_visible {
            let f_start = transform.translation + transform.forward() * axes.inner_offset;
            let r_start = transform.translation + transform.right() * axes.inner_offset;
            let u_start = transform.translation + transform.up() * axes.inner_offset;
            let f_end = f_start + transform.forward() * axes.size;
            let r_end = r_start + transform.right() * axes.size;
            let u_end = u_start + transform.up() * axes.size;
            lines.line_colored(f_start, f_end, 0., Color::BLUE);
            lines.line_colored(r_start, r_end, 0., Color::RED);
            lines.line_colored(u_start, u_end, 0., Color::GREEN);
        }
    }
}
