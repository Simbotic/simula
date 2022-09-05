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
    fn state(&self, input: &Res<Input<Self::InputType>>) -> ActionState;
}

#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub enum ActionState {
    Idle,
    Begin,
    InProgress,
    End,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ActionInput {
    Keyboard(InputKeyboard),
    Mouse(InputMouse),
    KeyboardMouse(InputKeyboard, InputMouse),
}

trait Action {
    fn state(&self) -> ActionState;
    // fn send(world: &mut World, input: ActionInput);
}

#[derive(SystemLabel)]
#[system_label(ignore_fields)]
pub struct CookieClick {
    pub state: ActionState,
    pub inputs: Vec<ActionInput>,
}

impl Default for CookieClick {
    fn default() -> Self {
        Self {
            state: ActionState::Idle,
            inputs: vec![],
        }
    }
}

impl CookieClick {
    fn action_setup(app: &mut App, inputs: &[ActionInput]) {
        let res = Self {
            state: ActionState::Idle,
            inputs: inputs.to_vec(),
        };
        app.insert_resource(res);
        app.add_event::<Self>();
        app.add_system_to_stage(CoreStage::PreUpdate, Self::run);
    }

    fn run(
        action: Res<Self>,
        mut event: EventWriter<Self>,
        keyboard: Res<Input<KeyCode>>,
        mouse_button: Res<Input<MouseButton>>,
    ) {
        for input in &action.inputs {
            let state = match input {
                ActionInput::Keyboard(input) => input.state(&keyboard),
                ActionInput::Mouse(input) => input.state(&mouse_button),
                _ => panic!("NOOOOOOO"),
            };
            if state != ActionState::Idle {
                event.send(CookieClick {
                    state,
                    inputs: vec![],
                });
            }
        }
    }
}

pub struct InputControlPlugin;

impl Plugin for InputControlPlugin {
    fn build(&self, app: &mut App) {
        // let mut list: Vec<Box<dyn InputActionType<InputType = dyn Eq + Hash>>> = vec![];

        let kb = ActionInput::Keyboard(InputKeyboard {
            key_code: KeyCode::A,
            shift: false,
            ctrl: false,
            alt: false,
        });

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

        CookieClick::action_setup(app, &[kb]);
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
