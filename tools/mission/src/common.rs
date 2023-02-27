use bevy::prelude::*;

pub trait Robot {
    fn get_energy(&self) -> f32;
    fn get_money(&self) -> u64;
    fn get_follow_ui(&self) -> Option<Entity>;
    fn set_energy(&mut self, energy: f32);
    fn set_money(&mut self, money: u64);
    fn set_follow_ui(&mut self, entity: Entity);
    fn starting_energy(&self) -> f32;
    fn rest_speed(&self) -> f32 {
        self.starting_energy() / 5.0
    }
}
