use bevy::{
    ecs::{query::WorldQuery},
    prelude::*,
};
use simula_mission::{
    agent::Agent,
    game::{Game, GameCreate},
};

#[derive(Debug)]
pub struct TicTacToe;

impl Plugin for TicTacToe {
    fn build(&self, app: &mut App) {
        app.add_system(start_game);
    }
}

#[derive(Debug, Component)]
pub struct VacantRole<const N: u8>;

#[derive(Debug, Component)]
pub struct ActiveRole<const N: u8> {
    pub agent: Entity,
}



// #[derive(WorldQuery)]
// #[world_query(derive(Debug))]
// struct GameQuery<T> {
//     games: &'static Game<T>,
// }

// type Q0 = Query<'world, 'state, (Entity, &GameCreate<TicTacToe>)>;

// ParamSet<(
//     Query<&mut Health, With<Enemy>>,
//     Query<&mut Health, With<Player>>,
// )>

fn start_game(mut commands: Commands, games: Query<(Entity, &GameCreate<TicTacToe>)>) {
    println!("Hello, world!");

    // let q0 = Query<(Entity, &GameCreate<TicTacToe>)>;
}
