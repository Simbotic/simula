use crate::{
    action::{ActionInputState, ActionState},
    InputChannel,
};
use bevy::prelude::*;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, Serialize, Deserialize, Hash)]
pub struct InputMouseButton {
    pub button: MouseButton,
}

impl ActionInputState for InputMouseButton {
    type InputType = MouseButton;

    fn state(&self, input_channel: &mut InputChannel<Self::InputType>) -> ActionState {
        if input_channel.input.just_pressed(self.button) {
            ActionState::Begin
        } else if input_channel.input.just_released(self.button) {
            ActionState::End
        } else if input_channel.input.pressed(self.button) {
            ActionState::InProgress
        } else {
            ActionState::Idle
        }
    }
}
