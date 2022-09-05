use crate::{ActionInputState, ActionState};
use bevy::prelude::*;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct InputKeyboard {
    pub key_code: KeyCode,
    pub shift: bool,
    pub ctrl: bool,
    pub alt: bool,
}

impl InputKeyboard {
    fn is_modified(&self, input: &Res<Input<KeyCode>>) -> bool {
        let mut is_modified = true;
        if self.shift {
            is_modified = is_modified && input.any_pressed([KeyCode::LShift, KeyCode::RShift]);
        }
        if self.ctrl {
            is_modified = is_modified && input.any_pressed([KeyCode::LControl, KeyCode::RControl]);
        }
        if self.alt {
            is_modified = is_modified && input.any_pressed([KeyCode::LAlt, KeyCode::RAlt]);
        }
        is_modified
    }
}

impl ActionInputState for InputKeyboard {
    type InputType = KeyCode;

    fn state(&self, input: &Res<Input<Self::InputType>>) -> ActionState {
        if self.is_modified(&input) && input.just_pressed(self.key_code) {
            ActionState::Begin
        } else if self.is_modified(&input) && input.just_released(self.key_code) {
            ActionState::End
        } else if self.is_modified(&input) && input.pressed(self.key_code) {
            ActionState::InProgress
        } else {
            ActionState::Idle
        }
    }
}
