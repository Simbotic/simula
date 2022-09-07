use crate::{
    action::{ActionInputState, ActionState},
    InputChannel,
};
use bevy::prelude::*;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, Serialize, Deserialize, Hash)]
pub struct InputKeyboard {
    pub key_code: KeyCode,
    pub shift: bool,
    pub ctrl: bool,
    pub alt: bool,
}

impl Default for InputKeyboard {
    fn default() -> Self {
        Self {
            key_code: KeyCode::Escape,
            shift: false,
            ctrl: false,
            alt: false,
        }
    }
}

impl InputKeyboard {
    fn is_modified(&self, input: &Input<KeyCode>) -> bool {
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

    fn state(&self, input_channel: &mut InputChannel<Self::InputType>) -> ActionState {
        if self.is_modified(&input_channel.input) && input_channel.input.just_pressed(self.key_code)
        {
            ActionState::Begin
        } else if self.is_modified(&input_channel.input)
            && input_channel.input.just_released(self.key_code)
        {
            ActionState::End
        } else if self.is_modified(&input_channel.input)
            && input_channel.input.pressed(self.key_code)
        {
            ActionState::InProgress
        } else {
            ActionState::Idle
        }
    }
}
