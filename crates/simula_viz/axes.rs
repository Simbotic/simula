use super::lines;
use bevy::prelude::*;

#[derive(Component)]
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
    pub visibility: Visibility,
    pub transform: Transform,
    pub global_transform: GlobalTransform,
}

impl Default for AxesBundle {
    fn default() -> Self {
        AxesBundle {
            axes: Default::default(),
            visibility: Visibility { is_visible: true },
            transform: Default::default(),
            global_transform: Default::default(),
        }
    }
}

pub fn system(mut lines: ResMut<lines::Lines>, query: Query<(&Axes, &GlobalTransform, &Visibility)>) {
    for (axes, transform, visibility) in query.iter() {
        if visibility.is_visible {
            let f_start = transform.translation + transform.forward() * axes.inner_offset;
            let r_start = transform.translation + transform.right() * axes.inner_offset;
            let u_start = transform.translation + transform.up() * axes.inner_offset;
            let f_end = f_start + transform.forward() * axes.size;
            let r_end = r_start + transform.right() * axes.size;
            let u_end = u_start + transform.up() * axes.size;
            lines.line_colored(f_start, f_end, Color::BLUE);
            lines.line_colored(r_start, r_end, Color::RED);
            lines.line_colored(u_start, u_end, Color::GREEN);
        }
    }
}
