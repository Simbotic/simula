use crate::action::{ActionInputState, ActionState};
use bevy::prelude::*;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
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

    fn state(&self, prev_state: ActionState, input: &mut Input<Self::InputType>) -> ActionState {

        // if input.clear_just_pressed(self.button) {
        //     if prev_state == ActionState::Idle {
        //         ActionState::Begin
        //     } else {
        //         panic!("Invalid state");
        //     }
        // } else {
        //     if prev_state == ActionState::Begin {
        //         ActionState::InProgress
        //     } else if prev_state == ActionState::InProgress {
        //         if input.pressed(self.button) {
        //             ActionState::InProgress
        //         } else {
        //             ActionState::End
        //         }
        //     } else {
        //         ActionState::Idle
        //     }
        // }

        if self.is_modified(&input) && input.clear_just_pressed(self.key_code) {
            if prev_state == ActionState::Idle {
                ActionState::Begin
            } else {
                panic!("Invalid state");
            }
        } else {
            if prev_state == ActionState::Begin {
                ActionState::InProgress
            } else if prev_state == ActionState::InProgress {
                if self.is_modified(&input) && input.pressed(self.key_code) {
                    ActionState::InProgress
                } else {
                    ActionState::End
                }
            } else {
                ActionState::Idle
            }
        }


        // if self.is_modified(&input) && input.just_pressed(self.key_code) {
        //     ActionState::Begin
        // } else if self.is_modified(&input) && input.just_released(self.key_code) {
        //     ActionState::End
        // } else if self.is_modified(&input) && input.pressed(self.key_code) {
        //     ActionState::InProgress
        // } else {
        //     ActionState::Idle
        // }
    }
}
