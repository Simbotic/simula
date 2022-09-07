use crate::{keyboard::InputKeyboard, mouse::InputMouseButton, InputChannel};
use bevy::prelude::*;
use bevy_egui::egui::util::hash;
use serde::{Deserialize, Serialize};
use std::hash::Hash;

pub(crate) trait ActionInputState {
    type InputType: Send + Sync + std::hash::Hash + Eq + 'static;
    fn state(&self, input_channel: &mut InputChannel<Self::InputType>) -> ActionState;
}

#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub enum ActionState {
    Idle,
    Begin,
    InProgress,
    End,
}

#[derive(Debug, Clone, Serialize, Deserialize, Hash)]
pub enum ActionInput {
    Keyboard(InputKeyboard),
    MouseButton(InputMouseButton),
    KeyboardMouseButton(InputKeyboard, InputMouseButton),
}

impl ActionInput {
    pub fn with_keycode(key_code: KeyCode) -> Self {
        Self::Keyboard(InputKeyboard {
            key_code,
            ..Default::default()
        })
    }

    pub fn with_mouse_button(button: MouseButton) -> Self {
        Self::MouseButton(InputMouseButton { button })
    }

    pub fn state(
        &self,
        action_input_id: u64,
        keyboard: &mut InputChannel<KeyCode>,
        mouse_button: &mut InputChannel<MouseButton>,
    ) -> Option<ActionState> {
        let state = match self {
            ActionInput::Keyboard(keyboard_input) => {
                if let Some(owner) = keyboard.owner {
                    if owner != action_input_id {
                        return None;
                    }
                }
                let state = keyboard_input.state(keyboard);
                if state == ActionState::Begin {
                    keyboard.owner = Some(action_input_id);
                } else if state == ActionState::Idle {
                    keyboard.owner = None;
                }
                state
            }
            ActionInput::MouseButton(mouse_button_input) => {
                if let Some(owner) = mouse_button.owner {
                    if owner != action_input_id {
                        return None;
                    }
                }
                let state = mouse_button_input.state(mouse_button);
                if state == ActionState::Begin {
                    mouse_button.owner = Some(action_input_id);
                } else if state == ActionState::Idle {
                    mouse_button.owner = None;
                }
                state
            }
            ActionInput::KeyboardMouseButton(keyboard_input, mouse_button_input) => {
                if let Some(owner) = mouse_button.owner {
                    if owner != action_input_id {
                        return None;
                    }
                }
                if keyboard.owner.is_some() {
                    return None;
                }

                let keyboard_state = keyboard_input.state(keyboard);
                let mouse_button_state = mouse_button_input.state(mouse_button);

                if (keyboard_state == ActionState::Begin
                    || keyboard_state == ActionState::InProgress)
                    && mouse_button_state == ActionState::Begin
                {
                    mouse_button.owner = Some(action_input_id);
                    ActionState::Begin
                } else if (keyboard_state == ActionState::End
                    && mouse_button_state == ActionState::InProgress)
                    || (keyboard_state == ActionState::InProgress
                        && mouse_button_state == ActionState::End)
                {
                    ActionState::End
                } else if keyboard_state == ActionState::InProgress
                    && mouse_button_state == ActionState::InProgress
                {
                    ActionState::InProgress
                } else {
                    mouse_button.owner = None;
                    ActionState::Idle
                }
            }
        };
        Some(state)
    }
}

pub struct Action<T>
where
    T: Struct + Default + SystemLabel + core::fmt::Debug + Hash,
{
    pub state: ActionState,
    pub label: T,
    inputs: Vec<ActionInput>,
}

impl<T> Action<T>
where
    T: Struct + Default + SystemLabel + core::fmt::Debug + Hash,
{
    pub fn add(app: &mut App, inputs: &[ActionInput], after: impl SystemLabel) {
        let res = Self {
            state: ActionState::Idle,
            label: T::default(),
            inputs: inputs.to_vec(),
        };
        app.insert_resource(res);
        app.add_event::<Self>();
        app.add_system_to_stage(
            CoreStage::PreUpdate,
            Self::run.label(T::default()).after(after),
        );
    }

    fn run(
        mut frame: Local<usize>,
        mut action: ResMut<Self>,
        mut event: EventWriter<Self>,
        mut keyboard: ResMut<InputChannel<KeyCode>>,
        mut mouse_button: ResMut<InputChannel<MouseButton>>,
    ) {
        let action = &mut *action;
        let prev_state = action.state;

        for input in action.inputs.iter_mut() {
            let action_input_id = hash((SystemLabel::type_id(&action.label), &input));
            if let Some(state) = input.state(action_input_id, &mut keyboard, &mut mouse_button) {
                action.state = state;
                if state != ActionState::Idle {
                    info!(
                        "Frame:{} Action [{:#?}] {:?} -> {:?}  Input: {:?} Id: {:x}",
                        *frame, action.label, prev_state, state, input, action_input_id
                    );
                    event.send(Self {
                        state,
                        label: T::default(),
                        inputs: vec![],
                    });
                }
            }
        }

        *frame += 1;
    }
}
