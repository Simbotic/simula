use super::lines::{Lines, LinesMaterial};
use bevy::{
    prelude::*,
    render::view::{ComputedVisibility, Visibility},
};

#[derive(Component, Reflect)]
#[reflect(Component)]
pub struct Grid {
    pub size: u32,
    pub divisions: u32,
    pub start_color: Color,
    pub end_color: Color,
}

impl Default for Grid {
    fn default() -> Self {
        Grid {
            size: 10,
            divisions: 10,
            start_color: Color::rgb(0.025, 0.02, 0.03),
            end_color: Color::rgb(0.025, 0.02, 0.03),
        }
    }
}

#[derive(Bundle, Default)]
pub struct GridBundle {
    pub grid: Grid,
    pub lines: Lines,
    pub transform: Transform,
    pub global_transform: GlobalTransform,
    pub visibility: Visibility,
    pub computed_visibility: ComputedVisibility,
}

fn add_lines(mut commands: Commands, query: Query<Entity, (With<Grid>, Without<Lines>)>) {
    for entity in query.iter() {
        commands.entity(entity).insert(Lines::default());
    }
}

fn update(mut query: Query<(&mut Lines, &Grid, &Visibility), With<Handle<LinesMaterial>>>) {
    for (mut lines, grid, visibility) in query.iter_mut() {
        match visibility {
            Visibility::Visible | Visibility::Inherited => {
                let center = grid.divisions / 2;
                let step = grid.size / grid.divisions;
                let half_size: i64 = (grid.size / 2).into();

                let i = 0..=grid.divisions;
                let k = -half_size..=half_size;

                if step == 0 {
                    continue;
                }

                for (i, k) in i.zip(k.step_by(step as usize)) {
                    let mut start_color = grid.start_color;
                    if i == center {
                        start_color = start_color + start_color * 0.3;
                    }

                    let x_a = Vec3::new(-half_size as f32, 0., k as f32);
                    let x_b = Vec3::new(half_size as f32, 0., k as f32);
                    lines.line_gradient(x_a, x_b, start_color, grid.end_color);

                    let z_a = Vec3::new(k as f32, 0., -half_size as f32);
                    let z_b = Vec3::new(k as f32, 0., half_size as f32);
                    lines.line_gradient(z_a, z_b, start_color, grid.end_color);
                }
            }
            Visibility::Hidden => {}
        }
    }
}

pub struct GridPlugin;

impl Plugin for GridPlugin {
    fn build(&self, app: &mut App) {
        app.register_type::<Grid>()
            .add_system(update)
            .add_system(add_lines);
    }
}
