use super::lines::{Lines, LinesMaterial};
use bevy::{
    prelude::*,
    render::view::{ComputedVisibility, Visibility},
};

#[derive(Reflect, Component)]
#[reflect(Component)]
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
    pub lines: Lines,
    pub material: LinesMaterial,
    pub transform: Transform,
    pub global_transform: GlobalTransform,
    pub visibility: Visibility,
    pub computed_visibility: ComputedVisibility,
}

impl Default for AxesBundle {
    fn default() -> Self {
        AxesBundle {
            axes: Default::default(),
            lines: Default::default(),
            material: Default::default(),
            transform: Default::default(),
            global_transform: Default::default(),
            visibility: Visibility::default(),
            computed_visibility: ComputedVisibility::default(),
        }
    }
}

fn system(mut query: Query<(&mut Lines, &Axes, &Visibility), With<LinesMaterial>>) {
    let transform = Transform::default();
    for (mut lines, axes, visibility) in query.iter_mut() {
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

pub struct AxesPlugin;

impl Plugin for AxesPlugin {
    fn build(&self, app: &mut App) {
        app.register_type::<Axes>().add_system(system);
    }
}
