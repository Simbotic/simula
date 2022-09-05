use crate::{keyboard::InputKeyboard, mouse::InputMouse};
use bevy::{input::InputSystem, prelude::*};
use serde::{Deserialize, Serialize};
use std::hash::Hash;

mod egui;
// pub mod inputmap;
pub mod keyboard;
pub mod mouse;

trait ActionInputState {
    type InputType: Send + Sync + Hash + Eq + 'static;

    fn pressed(&self, input: Res<Input<Self::InputType>>) -> bool;
    fn just_pressed(&self, input: Res<Input<Self::InputType>>) -> bool;
    fn just_released(&self, input: Res<Input<Self::InputType>>) -> bool;
}

trait ActionName {
    fn name(&self) -> String;
}

#[derive(Debug, Serialize, Deserialize)]
enum ActionInput {
    Keyboard(InputKeyboard),
    Mouse(InputMouse),
    KeyboardMouse(InputKeyboard, InputMouse),
}

struct ActionMap {
    name: String,
    help: String,
    input: ActionInput,
}

#[derive(Debug, Default)]
pub struct Action {
    pub pressed: bool,
    pub just_pressed: bool,
    pub just_released: bool,
}

trait InputMap {

    fn add_action(&mut self, name: T, input: ActionInput) {

    }
}

pub struct InputControlPlugin;

impl Plugin for InputControlPlugin {
    fn build(&self, app: &mut App) {
        // let mut list: Vec<Box<dyn InputActionType<InputType = dyn Eq + Hash>>> = vec![];

        let kb = keyboard::InputKeyboard {
            key_code: KeyCode::A,
            shift: false,
            ctrl: false,
            alt: false,
        };

        // list.push(Box::new(kb));

        app.init_resource::<egui::EguiBlockInputState>()
            .add_system_to_stage(
                CoreStage::PreUpdate,
                egui::egui_block_input.after(InputSystem),
            )
            .add_system_to_stage(
                CoreStage::PostUpdate,
                egui::egui_wants_input.after(egui::EguiSystem::ProcessOutput),
            )
            .add_startup_system(setup)
            .add_system(run);
    }
}

fn setup() {}

fn run(mouse_button: Res<Input<MouseButton>>, keyboard: Res<Input<KeyCode>>) {
    if keyboard.just_pressed(KeyCode::A) {
        println!("A");
    }
    if mouse_button.just_pressed(MouseButton::Left) {
        println!("Left");
    }
}
