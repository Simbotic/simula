use bevy::{input::InputSystem, prelude::*};
use bevy_egui::{EguiContext, EguiSystem};

pub struct InputControlPlugin;

impl Plugin for InputControlPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<EguiBlockInputState>()
            .add_system_to_stage(CoreStage::PreUpdate, egui_block_input.after(InputSystem))
            .add_system_to_stage(
                CoreStage::PostUpdate,
                egui_wants_input.after(EguiSystem::ProcessOutput),
            )
            .add_startup_system(setup)
            .add_system(run);
    }
}

// Check if Egui is blocking input
#[derive(Default)]
struct EguiBlockInputState {
    wants_keyboard_input: bool,
    wants_pointer_input: bool,
}

fn egui_wants_input(mut state: ResMut<EguiBlockInputState>, mut egui_context: ResMut<EguiContext>) {
    state.wants_keyboard_input = egui_context.ctx_mut().wants_keyboard_input();
    state.wants_pointer_input = egui_context.ctx_mut().wants_pointer_input();
}

fn egui_block_input(
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

fn setup() {}

fn run() {}
