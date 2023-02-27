use bevy::prelude::*;
use simula_video::VideoMaterial;

use crate::{common::Robot, spawn_robot_with_wallet};

pub const COP_STARTING_MONEY: u64 = 0;
pub const COP_STARTING_ENERGY: f32 = 500.0;
pub const COP_SPEED: f32 = 1.0;

#[derive(Component, Debug, Default, Clone, Copy)]
pub struct Cop {
    energy: f32,
    money: u64,
    follow_ui: Option<Entity>,
}

impl Robot for Cop {
    fn get_energy(&self) -> f32 {
        self.energy
    }

    fn get_money(&self) -> u64 {
        self.money
    }

    fn get_follow_ui(&self) -> Option<Entity> {
        self.follow_ui
    }

    fn set_energy(&mut self, energy: f32) {
        self.energy = energy;
    }

    fn set_money(&mut self, money: u64) {
        self.money = money;
    }

    fn set_follow_ui(&mut self, entity: Entity) {
        self.follow_ui = Some(entity);
    }

    fn starting_energy(&self) -> f32 {
        COP_STARTING_ENERGY
    }
}

pub fn cop_spawner(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut video_materials: ResMut<Assets<VideoMaterial>>,
    asset_server: Res<AssetServer>,
    query: Query<Entity, With<Cop>>,
    camera_query: Query<Entity, With<Camera>>,
) {
    if query.is_empty() {
        if let Ok(camera_entity) = camera_query.get_single() {
            spawn_robot_with_wallet(
                &mut commands,
                &mut meshes,
                &mut video_materials,
                &asset_server,
                &camera_entity,
                "robot_cop",
                COP_SPEED,
                &mut Cop {
                    energy: COP_STARTING_ENERGY,
                    money: COP_STARTING_MONEY,
                    follow_ui: None,
                },
            );
        }
    }
}
