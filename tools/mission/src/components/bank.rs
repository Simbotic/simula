use bevy::prelude::*;
use simula_core::ease::EaseFunction;
use simula_viz::{follow_ui::FollowUI, lookat::SmoothLookAt};

use crate::{common::Robot, ui};

pub const BANK_STARTING_MONEY: u64 = 50;
pub const BANK_STARTING_ENERGY: f32 = 500.0;
pub const BANK_MAX_MONEY: u64 = 500;

#[derive(Component, Debug, Default, Clone, Copy)]
pub struct Bank {
    money: u64,
    energy: f32,
    follow_ui: Option<Entity>,
}

impl Robot for Bank {
    fn get_money(&self) -> u64 {
        self.money
    }

    fn set_money(&mut self, money: u64) {
        self.money = money;
    }

    fn get_energy(&self) -> f32 {
        self.energy
    }

    fn set_energy(&mut self, energy: f32) {
        self.energy = energy;
    }

    fn get_follow_ui(&self) -> Option<Entity> {
        self.follow_ui
    }

    fn set_follow_ui(&mut self, entity: Entity) {
        self.follow_ui = Some(entity);
    }

    fn starting_energy(&self) -> f32 {
        BANK_STARTING_ENERGY
    }
}

pub fn bank_spawner(
    mut commands: Commands,
    asset_server: Res<AssetServer>,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    query: Query<Entity, With<Bank>>,
    camera_query: Query<Entity, With<Camera>>,
) {
    if query.is_empty() {
        if let Ok(camera_entity) = camera_query.get_single() {
            let texture_handle = asset_server.load("textures/mission/robot-bank.png");

            let material_handle = materials.add(StandardMaterial {
                base_color_texture: Some(texture_handle),
                alpha_mode: AlphaMode::Blend,
                unlit: true,
                ..default()
            });

            let texture_rotation =
                Quat::from_euler(EulerRot::YXZ, -std::f32::consts::FRAC_PI_3 * 0.0, 0.0, 0.0);
            let texture_position = Vec3::new(-2.0, 0.5, -1.0);

            let follow_ui_entity = commands
                .spawn(SpatialBundle {
                    transform: Transform::from_translation(Vec3::new(0.0, 1.0, 0.0)),
                    ..Default::default()
                })
                .insert(FollowUI {
                    min_distance: 0.1,
                    max_distance: 20.0,
                    min_height: -5.0,
                    max_height: 5.0,
                    max_view_angle: 45.0,
                    ..default()
                })
                .insert(SmoothLookAt {
                    target: Some(camera_entity),
                    yaw_ease: EaseFunction::SineInOut,
                    pitch_ease: EaseFunction::SineInOut,
                    ..default()
                })
                .insert(ui::RobotPanel)
                .insert(Name::new("FollowUI"))
                .id();

            let texture_entity = commands
                .spawn(SpatialBundle {
                    transform: Transform::from_translation(texture_position)
                        .with_rotation(texture_rotation),
                    ..default()
                })
                .with_children(|parent| {
                    parent
                        .spawn(MaterialMeshBundle {
                            mesh: meshes.add(Mesh::from(shape::Plane {
                                size: 1.0,
                                ..default()
                            })),
                            material: material_handle,
                            transform: Transform::from_rotation(Quat::from_euler(
                                EulerRot::YXZ,
                                0.0,
                                -std::f32::consts::FRAC_PI_2,
                                0.0,
                            )),
                            ..default()
                        })
                        .insert(Name::new("Texture: RenderTarget"));
                })
                .insert(Name::new("Robot: Bank"))
                .insert(SmoothLookAt {
                    target: Some(camera_entity),
                    yaw_ease: EaseFunction::SineInOut,
                    pitch_ease: EaseFunction::SineInOut,
                    max_yaw: 20.0,
                    ..default()
                })
                .insert(Bank {
                    money: BANK_STARTING_MONEY,
                    energy: BANK_STARTING_ENERGY,
                    follow_ui: Some(follow_ui_entity),
                })
                .push_children(&[follow_ui_entity])
                .id();

            commands
                .spawn(SpatialBundle { ..default() })
                .push_children(&[texture_entity]);
        }
    }
}