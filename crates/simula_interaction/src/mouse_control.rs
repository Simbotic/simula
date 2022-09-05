use bevy::{
    input::mouse::{MouseButtonInput, MouseMotion},
    prelude::*,
};

pub struct MouseControlPlugin;

impl Plugin for MouseControlPlugin {
    fn build(&self, app: &mut App) {
        app.add_startup_system(setup).add_system(run);
    }
}

#[derive(Component)]
pub struct MouseControlCamera;

fn setup() {}

fn run(
    // mut mouse_motion_events: EventReader<MouseMotion>,
    mut mouse_button_input_events: EventReader<MouseButtonInput>,
    // mut mouse_button_input: EventReader<MouseButtonInput>,
) {
    for evt in mouse_button_input_events.iter() {
        
    }
    // if mouse_button_input.pressed(MouseButton::Left) {
    //     mouse_button_input.
    // }
}
