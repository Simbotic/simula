use bevy::prelude::*;
use simula_video::VideoMaterial;

use crate::{common::Robot, cop::Cop, spawn_robot_with_wallet};

pub const ROBBER_STARTING_MONEY: u64 = 500;
pub const ROBBER_STARTING_ENERGY: u64 = 500;

#[derive(Component)]
pub struct RobberRest;

#[derive(Component)]
pub struct RobberRun;

#[derive(Component)]
pub struct RobberCaptured;

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
        // attempt to spawn a robber
        // make sure that we only have one camera in the scene
        if let Ok(camera_entity) = camera_query.get_single() {
            if let Ok(cop_transform) = cop_query.get_single() {
                let cop_rotation = cop_transform.rotation;
                // spawn a robber if the cop is sufficiently rotated
                if cop_rotation.y > 0.5 || cop_rotation.y < -0.5 {
                    let robber_entity = spawn_robot_with_wallet(
                        &mut commands,
                        &mut meshes,
                        &mut video_materials,
                        &asset_server,
                        &camera_entity,
                        "robot_robber",
                        1.5,
                        &mut Robber {
                            energy: ROBBER_STARTING_ENERGY,
                            money: ROBBER_STARTING_MONEY,
                            follow_ui: None,
                        },
                    );
                    commands.entity(robber_entity).insert(RobberRun);
                }
            }
        }
    }
}
