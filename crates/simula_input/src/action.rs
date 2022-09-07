use crate::{keyboard::InputKeyboard, mouse::InputMouseButton};
use bevy::prelude::*;
use serde::{Deserialize, Serialize};
use std::hash::Hash;

pub(crate) trait ActionInputState {
    type InputType: Send + Sync + Hash + Eq + 'static;
    fn state(&self, prev_state: ActionState, input: &mut Input<Self::InputType>) -> ActionState;
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
        prev_state: ActionState,
        keyboard: &mut Input<KeyCode>,
        mouse_button: &mut Input<MouseButton>,
    ) -> ActionState {
        match self {
            ActionInput::Keyboard(input) => input.state(prev_state, keyboard),
            ActionInput::MouseButton(input) => input.state(prev_state, mouse_button),
            ActionInput::KeyboardMouseButton(input_keyboard, input_mouse) => {
                let keyboard_state = input_keyboard.state(prev_state, keyboard);
                let mouse_button_state = input_mouse.state(prev_state, mouse_button);
                if keyboard_state == ActionState::Begin
                    && mouse_button_state == ActionState::InProgress
                {
                    ActionState::Begin
                } else if keyboard_state == ActionState::InProgress
                    && mouse_button_state == ActionState::Begin
                {
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
                    ActionState::Idle
                }
            }
        }
    }
}

pub struct Action<T>
where
    T: Struct + Default + SystemLabel + core::fmt::Debug,
{
    pub state: ActionState,
    pub label: T,
    inputs: Vec<ActionInput>,
    active_input: Option<u8>,
}

impl<T> Action<T>
where
    T: Struct + Default + SystemLabel + core::fmt::Debug,
{
    pub fn add(app: &mut App, inputs: &[ActionInput], after: impl SystemLabel) {
        let res = Self {
            state: ActionState::Idle,
            label: T::default(),
            inputs: inputs.to_vec(),
            active_input: None,
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
        mut keyboard: ResMut<Input<KeyCode>>,
        mut mouse_button: ResMut<Input<MouseButton>>,
    ) {
        *frame += 1;
        let mut valid_state = None;
        let prev_state = action.state;
        let action = &mut *action;

        for (idx, input) in action.inputs.iter_mut().enumerate() {
            if action.active_input.is_some() && action.active_input != Some(idx as u8) {
                continue;
            }
            let next_state = input.state(prev_state, &mut keyboard, &mut mouse_button);
            if next_state != ActionState::Idle {
                valid_state = Some(next_state);
                info!(
                    "Frame:{} Action [{:#?}] {:?} -> {:?}  Input: {:?}",
                    *frame, action.label, action.state, valid_state, input
                );
                action.active_input = Some(idx as u8);
                action.state = next_state;
                event.send(Self {
                    state: next_state,
                    label: T::default(),
                    inputs: vec![],
                    active_input: None,
                });
                break;
            }
        }

        if valid_state.is_none() {
            action.state = ActionState::Idle;
            action.active_input = None;
        }
    }
}
