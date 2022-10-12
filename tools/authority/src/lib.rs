use bevy::prelude::*;
use serde::{Deserialize, Serialize};
use simula_net::{replicate, Replicate};

pub struct NetAuthorityPlugin;

#[derive(Reflect, Default, Debug, Component, Serialize, Deserialize)]
#[reflect(Component)]
pub struct Authority {
    pub count: u32,
}

pub enum Command {
    SayHi,
    Ask(String),
}

pub enum Protocol {
    Authority,
    Command(Command),
}

impl Plugin for NetAuthorityPlugin {
    fn build(&self, app: &mut App) {
        app.register_type::<Authority>()
            .add_startup_system(setup)
            .add_system(run)
            .add_system(replicate::<Authority>);
    }
}

fn setup(mut _commands: Commands) {}

fn run(mut authorities: Query<&mut Authority, With<Replicate>>) {
    for mut authority in authorities.iter_mut() {
        authority.count += 1;
    }
}
