use bevy::prelude::*;

pub trait Robot {
    fn get_energy(&self) -> u64;
    fn get_money(&self) -> u64;
    fn get_follow_ui(&self) -> Option<Entity>;
    fn set_energy(&mut self, energy: u64);
    fn set_money(&mut self, money: u64);
    fn set_follow_ui(&mut self, entity: Entity);
}

#[derive(Component)]
pub struct RobotPanel;

#[derive(Component)]
pub struct Rotate {
    pub axis: Vec3,
    pub angle: f32,
}

#[derive(Component)]
pub struct CanRotate;
