use crate::{keyboard::InputKeyboard, mouse::InputMouseButton};
use bevy::prelude::*;
use serde::{Deserialize, Serialize};
use std::hash::Hash;

pub(crate) trait ActionInputState {
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
        keyboard: &Res<Input<KeyCode>>,
        mouse_button: &Res<Input<MouseButton>>,
    ) -> ActionState {
        match self {
            ActionInput::Keyboard(input) => input.state(keyboard),
            ActionInput::MouseButton(input) => input.state(mouse_button),
            ActionInput::KeyboardMouseButton(input_keyboard, input_mouse) => {
                let keyboard_state = input_keyboard.state(keyboard);
                let mouse_button_state = input_mouse.state(mouse_button);
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

#[derive(SystemLabel)]
#[system_label(ignore_fields)]
pub struct Action<T>
where
    T: Send + Sync + 'static,
{
    pub state: ActionState,
    inputs: Vec<ActionInput>,
    _phantom: std::marker::PhantomData<T>,
}

impl<T> Action<T>
where
    T: Send + Sync + 'static,
{
    pub fn add(app: &mut App, inputs: &[ActionInput]) {
        let res = Self {
            state: ActionState::Idle,
            inputs: inputs.to_vec(),
            _phantom: std::marker::PhantomData::default(),
        };
        app.insert_resource(res);
        app.add_event::<Self>();
        app.add_system_to_stage(CoreStage::PreUpdate, Self::run);
    }

    fn run(
        mut action: ResMut<Self>,
        mut event: EventWriter<Self>,
        keyboard: Res<Input<KeyCode>>,
        mouse_button: Res<Input<MouseButton>>,
    ) {
        action.state = ActionState::Idle;
        let mut next_state = None;
        for input in &action.inputs {
            let state = input.state(&keyboard, &mouse_button);
            if state != ActionState::Idle {
                next_state = Some(state);
                break;
            }
        }
        if let Some(next_state) = next_state {
            action.state = next_state;
            event.send(Self {
                state: next_state,
                inputs: vec![],
                _phantom: std::marker::PhantomData::default(),
            });
        }
    }
}
