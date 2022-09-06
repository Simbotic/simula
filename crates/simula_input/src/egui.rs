use bevy::prelude::*;
pub use bevy_egui::{EguiContext, EguiSystem};

#[derive(Default)]
pub struct EguiBlockInputState {
    wants_keyboard_input: bool,
    wants_pointer_input: bool,
}

pub fn egui_wants_input(
    mut state: ResMut<EguiBlockInputState>,
    mut egui_context: ResMut<EguiContext>,
) {
    state.wants_keyboard_input = egui_context.ctx_mut().wants_keyboard_input();
    state.wants_pointer_input = egui_context.ctx_mut().wants_pointer_input();
}

pub fn egui_block_input(
    state: Res<EguiBlockInputState>,
    mut keys: ResMut<Input<KeyCode>>,
    mut mouse_buttons: ResMut<Input<MouseButton>>,
) {
    if state.wants_keyboard_input {
        keys.reset_all();
    }
    if state.wants_pointer_input {
        mouse_buttons.reset_all();
    }
}
