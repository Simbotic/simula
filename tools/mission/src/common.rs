use bevy::prelude::*;

pub trait Robot {
    fn get_energy(&self) -> u64;
    fn get_money(&self) -> u64;
    fn get_follow_ui(&self) -> Option<Entity>;
    fn set_energy(&mut self, energy: u64);
    fn set_money(&mut self, money: u64);
    fn set_follow_ui(&mut self, entity: Entity);
    fn starting_energy(&self) -> u64;
    fn rest_speed(&self) -> u64 {
        self.starting_energy() / 25_u64
    }
}
