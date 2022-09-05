use crate::ActionInputState;
use bevy::prelude::*;
use serde::{Serialize, Deserialize};

#[derive(Debug, Serialize, Deserialize)]
pub struct InputMouse {
    pub button: MouseButton,
}

impl ActionInputState for InputMouse {
    type InputType = MouseButton;

    fn pressed(&self, input: Res<Input<MouseButton>>) -> bool {
        input.pressed(self.button)
    }

    fn just_pressed(&self, input: Res<Input<MouseButton>>) -> bool {
        input.just_pressed(self.button)
    }

    fn just_released(&self, input: Res<Input<MouseButton>>) -> bool {
        input.just_released(self.button)
    }
}
