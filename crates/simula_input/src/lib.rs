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
    fn state(&self, input: Res<Input<Self::InputType>>) -> ActionState;
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub enum ActionState {
    Idle,
    Begin,
    InProgress,
    End,
}

#[derive(Debug, Serialize, Deserialize)]
enum ActionInput {
    Keyboard(InputKeyboard),
    Mouse(InputMouse),
    KeyboardMouse(InputKeyboard, InputMouse),
}

// fn to_action_state<T>(action_input: ActionInput, input: Res<Input<T>>) -> ActionState
// where
//     T: Send + Sync + Hash + Eq + 'static,
// {
//     match action_input {
//         ActionInput::Keyboard(action) => action.state(input),
//     }
// }

// struct ActionMap {
//     name: String,
//     help: String,
//     input: ActionInput,
// }

// #[derive(Debug, Default)]
// pub struct Action {
//     pub pressed: bool,
//     pub on_begin: bool,
//     pub on_end: bool,
// }
// add_action::<>

// trait InputMap {
//     fn add_action(name: T, input: ActionInput) {}
// }

// use proc_macro::TokenStream;

// #[proc_macro_derive(MyMacroHere)]
// pub fn my_macro_here_derive(input: TokenStream) -> TokenStream {
//     let expanded = quote! {
//         // ...
//     };
//     TokenStream::from(expanded)
// }

trait Action {
    fn state(&self) -> ActionState;
    // fn send(world: &mut World, input: ActionInput);
}

#[derive(Deref, SystemLabel)]
#[system_label(ignore_fields)]
pub struct CookieClick {
    state: ActionState,
}

impl CookieClick {
    fn action_setup(app: &mut App) {
        app.add_event::<Self>();
        app.add_system_to_stage(CoreStage::PreUpdate, Self::run);
    }

    fn run(
        mouse_button: Res<Input<MouseButton>>,
        keyboard: Res<Input<KeyCode>>,
        mut event: EventWriter<Self>,
    ) {
        if mouse_button.just_pressed(MouseButton::Left) {
            println!("Left");
            event.send(CookieClick {
                state: ActionState::Begin,
            });
        }
        if mouse_button.just_released(MouseButton::Left) {
            println!("Left");
            event.send(CookieClick {
                state: ActionState::End,
            });
        }
    }
}

impl Action for CookieClick {
    fn state(&self) -> ActionState {
        self.state
    }
}

fn add_action<T: Action>(app: &mut App, input: ActionInput) {
    app.add_event::<CookieClick>();
}

// struct InputMaps {
//     input_maps: Vec<>,
// }

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

        let mut maps = vec![];
        maps.push(());

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

        CookieClick::action_setup(app);
    }
}

fn setup() {}

fn run(mut evt: EventReader<CookieClick>) {
    for evt in evt.iter() {
        println!("CookieClick {:?}", evt.state);
    }
}

#[cfg(test)]
mod tests {
    #[test]
    fn it_works() {
        assert_eq!(2 + 2, 4);
    }
}