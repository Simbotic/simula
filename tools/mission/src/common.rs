use bevy::prelude::*;

pub trait Robot {
    fn get_energy(&self) -> u64;
    fn get_money(&self) -> u64;
    fn get_follow_ui(&self) -> Option<Entity>;
    fn set_energy(&mut self, energy: u64);
    fn set_money(&mut self, money: u64);
    fn set_follow_ui(&mut self, entity: Entity);
}

#[derive(Component, Reflect, Default)]
#[reflect(Component)]
pub struct Movement {
    pub points: Vec<Vec3>,
    pub duration: f32,
    pub elapsed: f32,
    pub direction: Vec3,
    pub index: usize,
}
