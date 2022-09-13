pub use action::Action;
pub use axis::ActionAxis;
use bevy::input::{
    gamepad::{Gamepad, GamepadAxis, GamepadButton},
    keyboard::{KeyCode, KeyboardInput},
    mouse::{MouseButton, MouseButtonInput, MouseMotion, MouseWheel},
    ButtonState,
};
use bevy::prelude::*;
use bevy_egui::{EguiContext, EguiSystem};
use std::fmt::Debug;
use std::hash::Hash;

pub mod action;
pub mod axis;

#[derive(Component)]
pub struct MainActionInput;

#[derive(Debug, PartialEq, Eq, Clone, Hash, SystemLabel)]
pub struct ActionPlugin;

impl Plugin for ActionPlugin {
    fn build(&self, app: &mut App) {
        app.add_system_to_stage(
            CoreStage::PreUpdate,
            keyboard_action_system
                .after(EguiSystem::ProcessInput)
                .before(EguiSystem::BeginFrame),
        )
        .add_system_to_stage(
            CoreStage::PreUpdate,
            mouse_button_action_system
                .after(EguiSystem::ProcessInput)
                .before(EguiSystem::BeginFrame),
        )
        .add_system_to_stage(
            CoreStage::PreUpdate,
            mouse_axis_system
                .after(EguiSystem::ProcessInput)
                .before(EguiSystem::BeginFrame),
        )
        .add_startup_system(setup);
    }
}

fn setup(mut commands: Commands) {
    commands
        .spawn()
        .insert(MainActionInput)
        .insert(Action::<KeyCode>::default())
        .insert(Action::<MouseButton>::default())
        .insert(ActionAxis::<MouseAxis>::default());
}

pub fn print_all_actions(
    keyboard_actions: Query<&Action<KeyCode>>,
    mouse_button_actions: Query<&Action<MouseButton>>,
    mouse_axis_actions: Query<&ActionAxis<MouseAxis>>,
) {
    for action in keyboard_actions.iter() {
        println!("{:?}", action);
    }
    for action in mouse_button_actions.iter() {
        println!("{:?}", action);
    }
    for action in mouse_axis_actions.iter() {
        println!("{:?}", action);
    }
}

pub fn keyboard_action_system(
    mut egui_context: ResMut<EguiContext>,
    mut keyboard_input_events: EventReader<KeyboardInput>,
    mut keyboard_actions: Query<&mut Action<KeyCode>>,
) {
    if egui_context.ctx_mut().wants_keyboard_input() {
        for mut action in keyboard_actions.iter_mut() {
            action.reset_all();
        }
        return;
    }
    for mut action in keyboard_actions.iter_mut() {
        action.clear();
    }

    for event in keyboard_input_events.iter() {
        if let KeyboardInput {
            key_code: Some(key_code),
            state,
            ..
        } = event
        {
            for mut action in keyboard_actions.iter_mut() {
                match state {
                    ButtonState::Pressed => action.enter(*key_code),
                    ButtonState::Released => action.exit(*key_code),
                }
            }
        }
    }
}

pub fn mouse_button_action_system(
    mut egui_context: ResMut<EguiContext>,
    mut mouse_button_input_events: EventReader<MouseButtonInput>,
    mut mouse_button_actions: Query<&mut Action<MouseButton>>,
) {
    if egui_context.ctx_mut().wants_pointer_input() {
        for mut action in mouse_button_actions.iter_mut() {
            action.reset_all();
        }
        return;
    }
    for mut action in mouse_button_actions.iter_mut() {
        action.clear();
    }
    for event in mouse_button_input_events.iter() {
        for mut action in mouse_button_actions.iter_mut() {
            match event.state {
                ButtonState::Pressed => action.enter(event.button),
                ButtonState::Released => action.exit(event.button),
            }
        }
    }
}

pub fn mouse_axis_system(
    mut egui_context: ResMut<EguiContext>,
    mut mouse_motion_input_events: EventReader<MouseMotion>,
    mut mouse_wheel_input_events: EventReader<MouseWheel>,
    mut mouse_axis_actions: Query<&mut ActionAxis<MouseAxis>>,
) {
    if egui_context.ctx_mut().wants_pointer_input() {
        return;
    }
    for mut action_axis in mouse_axis_actions.iter_mut() {
        action_axis.set(MouseAxis::X, 0.);
        action_axis.set(MouseAxis::Y, 0.);
    }
    for event in mouse_motion_input_events.iter() {
        for mut action_axis in mouse_axis_actions.iter_mut() {
            action_axis.set(MouseAxis::X, event.delta.x);
            action_axis.set(MouseAxis::Y, event.delta.y);
        }
    }

    for event in mouse_wheel_input_events.iter() {
        for mut action_axis in mouse_axis_actions.iter_mut() {
            action_axis.set(MouseAxis::Z, event.y);
        }
    }
}

#[derive(Debug)]
pub enum ActionMapButton {
    Keyboard(KeyCode),
    MouseButton(MouseButton),
    Gamepad(Gamepad, GamepadButton),
}

impl Into<ActionMapButton> for KeyCode {
    fn into(self) -> ActionMapButton {
        ActionMapButton::Keyboard(self)
    }
}

impl Into<ActionMapButton> for MouseButton {
    fn into(self) -> ActionMapButton {
        ActionMapButton::MouseButton(self)
    }
}

#[derive(Debug)]
pub struct ActionMapInput<T> {
    pub action: T,
    pub button: ActionMapButton,
    pub ctrl: bool,
    pub shift: bool,
    pub alt: bool,
}

#[derive(Debug, Deref, DerefMut, Component)]
pub struct ActionMap<T>(Vec<ActionMapInput<T>>);

impl<T> Default for ActionMap<T> {
    fn default() -> Self {
        Self(Vec::default())
    }
}

pub fn action_map<T, W>(
    keyboard_actions: Query<&Action<KeyCode>, With<MainActionInput>>,
    mouse_button_actions: Query<&Action<MouseButton>, With<MainActionInput>>,
    mut actions: Query<&mut Action<T>, With<W>>,
    action_maps: Query<&ActionMap<T>, With<W>>,
) where
    T: Debug + Send + Sync + Hash + Eq + Copy + Clone + 'static,
    W: Component,
{
    let keyboard_action = keyboard_actions.single();
    let mouse_button_action = mouse_button_actions.single();
    let mut action = actions.single_mut();

    for action_map in action_maps.single().iter() {
        let mut is_modified = true;
        if action_map.ctrl {
            is_modified = is_modified && keyboard_action.on(KeyCode::LControl);
        } else {
            is_modified = is_modified && !keyboard_action.on(KeyCode::LControl);
        }
        if action_map.shift {
            is_modified = is_modified && keyboard_action.on(KeyCode::LShift);
        } else {
            is_modified = is_modified && !keyboard_action.on(KeyCode::LShift);
        }
        if action_map.alt {
            is_modified = is_modified && keyboard_action.on(KeyCode::LAlt);
        } else {
            is_modified = is_modified && !keyboard_action.on(KeyCode::LAlt);
        }

        // Handle on_enter
        if is_modified {
            match action_map.button {
                ActionMapButton::Keyboard(key_code) => {
                    if keyboard_action.on_enter(key_code) {
                        if !action.on(action_map.action) {
                            action.enter(action_map.action);
                        }
                    }
                }
                ActionMapButton::MouseButton(mouse_button) => {
                    if mouse_button_action.on_enter(mouse_button) {
                        if !action.on(action_map.action) {
                            action.enter(action_map.action);
                        }
                    }
                }
                _ => panic!("Not implemented"),
            }
        }

        // Handle on_exit
        match action_map.button {
            ActionMapButton::Keyboard(key_code) => {
                if keyboard_action.on_exit(key_code) {
                    if action.on(action_map.action) {
                        action.exit(action_map.action);
                    }
                }
            }
            ActionMapButton::MouseButton(mouse_button) => {
                if mouse_button_action.on_exit(mouse_button) {
                    if action.on(action_map.action) {
                        action.exit(action_map.action);
                    }
                }
            }
            _ => panic!("Not implemented"),
        }
    }
}

#[derive(Debug, Hash, Eq, PartialEq, Copy, Clone)]
pub enum MouseAxis {
    X,
    Y,
    Z,
}

#[derive(Debug)]
pub enum AxisMapSource {
    Keyboard {
        positive: KeyCode,
        negative: KeyCode,
    },
    MouseAxis(MouseAxis),
    GamepadAxis(Gamepad, GamepadAxis),
}

impl Into<AxisMapSource> for MouseAxis {
    fn into(self) -> AxisMapSource {
        AxisMapSource::MouseAxis(self)
    }
}

#[derive(Debug)]
pub struct AxisMapInput<T> {
    pub axis: T,
    pub source: AxisMapSource,
}

#[derive(Debug, Deref, DerefMut, Component)]
pub struct AxisMap<T>(Vec<AxisMapInput<T>>);

impl<T> Default for AxisMap<T> {
    fn default() -> Self {
        Self(Vec::default())
    }
}

pub fn axis_map<T, W>(
    keyboard_actions: Query<&Action<KeyCode>, With<MainActionInput>>,
    mouse_axis_actions: Query<&ActionAxis<MouseAxis>, With<MainActionInput>>,
    mut axes: Query<&mut ActionAxis<T>, With<W>>,
    axis_maps: Query<&AxisMap<T>, With<W>>,
) where
    T: Debug + Send + Sync + Hash + Eq + Copy + Clone + 'static,
    W: Component,
{
    let keyboard_action = keyboard_actions.single();
    let mouse_axis_action = mouse_axis_actions.single();
    let mut axis = axes.single_mut();

    for axis_map in axis_maps.single().iter() {
        match axis_map.source {
            AxisMapSource::Keyboard { positive, negative } => {
                let mut value = 0.0;
                if keyboard_action.on(positive) {
                    value += 1.0;
                }
                if keyboard_action.on(negative) {
                    value -= 1.0;
                }
                axis.set(axis_map.axis, value);
            }
            AxisMapSource::MouseAxis(mouse_axis) => {
                if let Some(value) = mouse_axis_action.get(mouse_axis) {
                    axis.set(axis_map.axis, value);
                }
            }
            _ => panic!("Not implemented"),
        }
    }
}
