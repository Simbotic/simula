use bevy::{
    input::{
        keyboard::KeyboardInput,
        mouse::{MouseButtonInput, MouseMotion},
    },
    prelude::*,
};


// enum InputAction {
//     Action()
// }


// pub enum InputActionType {
//     MouseButton(MouseButton),
//     Keyboard(InputActionKeyboard),
// }

pub enum InputValueType {
    MouseMotionX,
    MouseMotionY,
    MouseWheel,
}
