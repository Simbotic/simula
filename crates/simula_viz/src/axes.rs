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

#[derive(Bundle, Default)]
pub struct AxesBundle {
    pub axes: Axes,
    pub lines: Lines,
    pub transform: Transform,
    pub global_transform: GlobalTransform,
    pub visibility: Visibility,
    pub computed_visibility: ComputedVisibility,
}

fn add_lines(mut commands: Commands, query: Query<Entity, (With<Axes>, Without<Lines>)>) {
    for entity in query.iter() {
        commands.entity(entity).insert(Lines::default());
    }
}

fn update(mut query: Query<(&mut Lines, &Axes, &Visibility), With<Handle<LinesMaterial>>>) {
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
        app.register_type::<Axes>()
            .add_system(update)
            .add_system(add_lines);
    }
}
