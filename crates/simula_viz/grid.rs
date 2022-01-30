use super::lines::{Lines, LinesMaterial};
use bevy::{
    prelude::*,
    render::view::{ComputedVisibility, Visibility},
};

#[derive(Component)]
pub struct Grid {
    pub size: u32,
    pub divisions: u32,
    pub color: Color,
}

impl Default for Grid {
    fn default() -> Self {
        Grid {
            size: 10,
            divisions: 10,
            color: Color::rgb(0.025, 0.02, 0.03),
        }
    }
}

#[derive(Bundle)]
pub struct GridBundle {
    pub grid: Grid,
    pub lines: Lines,
    pub material: LinesMaterial,
    pub transform: Transform,
    pub global_transform: GlobalTransform,
    pub visibility: Visibility,
    pub computed_visibility: ComputedVisibility,
}

impl Default for GridBundle {
    fn default() -> Self {
        GridBundle {
            grid: Default::default(),
            lines: Default::default(),
            material: Default::default(),
            transform: Default::default(),
            global_transform: Default::default(),
            visibility: Visibility::default(),
            computed_visibility: ComputedVisibility::default(),
        }
    }
}

pub fn system(mut query: Query<(&mut Lines, &Grid, &Visibility), With<LinesMaterial>>) {
    for (mut lines, grid, visibility) in query.iter_mut() {
        if visibility.is_visible {
            let center = grid.divisions / 2;
            let step = grid.size / grid.divisions;
            let half_size: i64 = (grid.size / 2).into();

            let i = 0..=grid.divisions;
            let k = -half_size..=half_size;

            for (i, k) in i.zip(k.step_by(step as usize)) {
                let mut color = grid.color;
                if i == center {
                    color = color + color * 0.3;
                }

                let x_a = Vec3::new(-half_size as f32, 0., k as f32);
                let x_b = Vec3::new(half_size as f32, 0., k as f32);
                lines.line_colored(x_a, x_b, color);

                let z_a = Vec3::new(k as f32, 0., -half_size as f32);
                let z_b = Vec3::new(k as f32, 0., half_size as f32);
                lines.line_colored(z_a, z_b, color);
            }
        }
    }
}
