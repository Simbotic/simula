use bevy::prelude::*;
use simula_video::VideoMaterial;

use crate::{common::Robot, components::cop::Cop, spawn_robot_with_wallet};

pub const ROBBER_STARTING_MONEY: u64 = 100;
pub const ROBBER_STARTING_ENERGY: u64 = 500;
pub const ROBBER_SPEED: f32 = 0.75;

#[derive(Component, Debug, Default, Clone, Copy)]
pub struct Robber {
    energy: u64,
    money: u64,
    follow_ui: Option<Entity>,
}

impl Robot for Robber {
    fn get_energy(&self) -> u64 {
        self.energy
    }

    fn get_money(&self) -> u64 {
        self.money
    }

    fn get_follow_ui(&self) -> Option<Entity> {
        self.follow_ui
    }

    fn set_energy(&mut self, energy: u64) {
        self.energy = energy;
    }

    fn set_money(&mut self, money: u64) {
        self.money = money;
    }

    fn set_follow_ui(&mut self, entity: Entity) {
        self.follow_ui = Some(entity);
    }

    fn starting_energy(&self) -> u64 {
        ROBBER_STARTING_ENERGY
    }
}

pub fn robber_spawner(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut video_materials: ResMut<Assets<VideoMaterial>>,
    asset_server: Res<AssetServer>,
    query: Query<Entity, With<Robber>>,
    cop_query: Query<&Transform, With<Cop>>,
    camera_query: Query<Entity, With<Camera>>,
) {
    if query.is_empty() {
        if let Ok(camera_entity) = camera_query.get_single() {
            if let Ok(cop_transform) = cop_query.get_single() {
                let cop_translation = cop_transform.translation;
                // spawn a robber if the cop is sufficiently rotated
                if cop_translation.x > 1.0 && cop_translation.x > -1.0 {
                    spawn_robot_with_wallet(
                        &mut commands,
                        &mut meshes,
                        &mut video_materials,
                        &asset_server,
                        &camera_entity,
                        "robot_robber",
                        ROBBER_SPEED,
                        &mut Robber {
                            energy: ROBBER_STARTING_ENERGY,
                            money: ROBBER_STARTING_MONEY,
                            follow_ui: None,
                        },
                    );
                }
            }
        }
    }
}
