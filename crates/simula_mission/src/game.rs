use bevy::prelude::*;

pub trait Player: Component {
    type Game: Game;

    fn game(&self) -> Entity;
}

pub trait Game: Component {
    type Player: Player;

    fn players(&self) -> Vec<Entity>;
    fn max_players(&self) -> usize;
}



