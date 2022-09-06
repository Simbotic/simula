use crate::action::{ActionInputState, ActionState};
use bevy::prelude::*;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct InputMouseButton {
    pub button: MouseButton,
}

impl ActionInputState for InputMouseButton {
    type InputType = MouseButton;

    fn state(&self, input: &Res<Input<Self::InputType>>) -> ActionState {
        if input.just_pressed(self.button) {
            ActionState::Begin
        } else if input.just_released(self.button) {
            ActionState::End
        } else if input.pressed(self.button) {
            ActionState::InProgress
        } else {
            ActionState::Idle
        }
    }
}
