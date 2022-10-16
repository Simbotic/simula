use bevy::prelude::*;

#[derive(Debug, Component)]
pub struct Game<T> {
    phantom: std::marker::PhantomData<T>,
}

#[derive(Component)]
pub struct GameCreate<T> {
    phantom: std::marker::PhantomData<T>,
}
