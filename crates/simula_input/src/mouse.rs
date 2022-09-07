use crate::action::{ActionInputState, ActionState};
use bevy::prelude::*;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub struct InputMouseButton {
    pub button: MouseButton,
}

impl ActionInputState for InputMouseButton {
    type InputType = MouseButton;

    fn state(&self, prev_state: ActionState, input: &mut Input<Self::InputType>) -> ActionState {
        if input.clear_just_pressed(self.button) {
            if prev_state == ActionState::Idle {
                ActionState::Begin
            } else {
                panic!("Invalid state");
            }
        } else {
            if prev_state == ActionState::Begin {
                ActionState::InProgress
            } else if prev_state == ActionState::InProgress {
                if input.pressed(self.button) {
                    ActionState::InProgress
                } else {
                    ActionState::End
                }
            } else {
                ActionState::Idle
            }
        }

        // if input.just_pressed(self.button) {
        //     ActionState::Begin
        // } else if input.just_released(self.button) {
        //     ActionState::End
        // } else if input.pressed(self.button) {
        //     ActionState::InProgress
        // } else {
        //     ActionState::Idle
        // }
    }
}
