pub use action::Action;
pub use axis::ActionAxis;
use bevy::input::{
    gamepad::{Gamepad, GamepadAxis, GamepadButton},
    keyboard::{KeyCode, KeyboardInput},
    mouse::{MouseButton, MouseButtonInput, MouseMotion, MouseScrollUnit, MouseWheel},
    ButtonState,
};
use bevy::{
    prelude::*,
    reflect::FromReflect,
    utils::{HashMap, HashSet},
};
use bevy_egui::{EguiContexts, EguiSet};
use std::fmt::Debug;
use std::hash::Hash;
use touch_sides::*;

pub mod action;
pub mod axis;
pub mod touch_sides;

#[derive(Default, Component, Reflect)]
#[reflect(Component)]
pub struct MainActionInput;

#[derive(Debug, PartialEq, Eq, Clone, Hash, SystemSet)]
pub struct ActionPlugin;

#[derive(SystemSet, Debug, Clone, PartialEq, Eq, Hash)]
pub enum ActionStage {
    PreUpdate,
    Update,
}

impl Plugin for ActionPlugin {
    fn build(&self, app: &mut App) {
        app.register_type::<MainActionInput>()
            .register_type::<Action<KeyCode>>()
            .register_type::<Action<MouseButton>>()
            .register_type::<ActionAxis<MouseAxis>>()
            .register_type::<HashSet<KeyCode>>()
            .register_type::<HashSet<MouseButton>>()
            .register_type::<HashMap<MouseAxis, f32>>()
            .add_plugin(TouchSidesPlugin)
            .configure_set(
                ActionStage::PreUpdate
                    .after(EguiSet::ProcessInput)
                    .before(EguiSet::BeginFrame)
                    .in_base_set(CoreSet::PreUpdate),
            )
            .configure_set(ActionStage::Update.in_base_set(CoreSet::Update))
            .add_systems(
                (
                    keyboard_action_system,
                    mouse_button_action_system,
                    mouse_axis_system,
                )
                    .chain()
                    .in_set(ActionStage::PreUpdate),
            )
            .add_startup_system(setup);
    }
}

fn setup(mut commands: Commands) {
    commands
        .spawn_empty()
        .insert(MainActionInput)
        .insert(Action::<KeyCode>::default())
        .insert(Action::<MouseButton>::default())
        .insert(ActionAxis::<MouseAxis>::default())
        .insert(Action::<TouchSide>::default())
        .insert(ActionAxis::<TouchSideAxis>::default())
        .insert(Name::new("Main: Action Input"));
}

pub fn print_all_actions(
    keyboard_actions: Query<&Action<KeyCode>>,
    mouse_button_actions: Query<&Action<MouseButton>>,
    mouse_axis_actions: Query<&ActionAxis<MouseAxis>>,
    touch_axis_actions: Query<&ActionAxis<TouchSideAxis>>,
    touch_side_actions: Query<&Action<TouchSide>>,
) {
    for action in keyboard_actions.iter() {
        info!("{:?}", action);
    }
    for action in mouse_button_actions.iter() {
        info!("{:?}", action);
    }
    for action in mouse_axis_actions.iter() {
        info!("{:?}", action);
    }
    for action in touch_axis_actions.iter() {
        info!("{:?}", action);
    }
    for action in touch_side_actions.iter() {
        info!("{:?}", action);
    }
}

pub fn keyboard_action_system(
    mut egui_context: EguiContexts,
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
    mut egui_context: EguiContexts,
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

const LINE_TO_PIXEL_RATIO: f32 = 0.1;

pub fn mouse_axis_system(
    mut egui_context: EguiContexts,
    mut mouse_motion_input_events: EventReader<MouseMotion>,
    mut mouse_wheel_input_events: EventReader<MouseWheel>,
    mut mouse_axis_actions: Query<&mut ActionAxis<MouseAxis>>,
) {
    if egui_context.ctx_mut().wants_pointer_input() {
        debug!("Egui wants pointer input");
        return;
    }
    let mut exy = Vec2::new(0., 0.);

    for mut action_axis in mouse_axis_actions.iter_mut() {
        action_axis.set(MouseAxis::X, 0.);
        action_axis.set(MouseAxis::Y, 0.);
        action_axis.set(MouseAxis::Z, 0.);
    }
    for event in mouse_motion_input_events.iter() {
        exy += event.delta * 0.01;
    }
    for mut action_axis in mouse_axis_actions.iter_mut() {
        action_axis.set(MouseAxis::X, exy.x);
        action_axis.set(MouseAxis::Y, exy.y);
    }
    let mut ez = 0.;
    for event in mouse_wheel_input_events.iter() {
        for mut action_axis in mouse_axis_actions.iter_mut() {
            ez += event.y
                * match event.unit {
                    MouseScrollUnit::Line => 1.0,
                    MouseScrollUnit::Pixel => LINE_TO_PIXEL_RATIO,
                };
            action_axis.set(MouseAxis::Z, ez);
        }
    }
}

#[derive(Debug, Clone, Reflect, FromReflect)]
pub enum ActionMapButton {
    Keyboard(KeyCode),
    MouseButton(MouseButton),
    Gamepad(Gamepad, GamepadButton),
    TouchSide(TouchSide),
}

impl From<KeyCode> for ActionMapButton {
    fn from(key_code: KeyCode) -> Self {
        ActionMapButton::Keyboard(key_code)
    }
}

impl From<MouseButton> for ActionMapButton {
    fn from(button: MouseButton) -> Self {
        ActionMapButton::MouseButton(button)
    }
}

impl From<TouchSide> for ActionMapButton {
    fn from(touch_side: TouchSide) -> Self {
        ActionMapButton::TouchSide(touch_side)
    }
}

#[derive(Debug, Clone, Reflect, FromReflect)]
pub struct ActionMapInput<T>
where
    T: Reflect + Clone + Debug + Eq + Hash + Send + Sync + 'static,
{
    pub action: T,
    pub button: ActionMapButton,
    pub ctrl: bool,
    pub shift: bool,
    pub alt: bool,
}

#[derive(Default, Debug, Clone, Deref, DerefMut, Component, Reflect, FromReflect)]
#[reflect(Component)]
pub struct ActionMap<T>
where
    T: Default + Reflect + FromReflect + Clone + Debug + Eq + Hash + Send + Sync + 'static,
{
    map: Vec<ActionMapInput<T>>,
}

pub fn action_map<T, W>(
    keyboard_actions: Query<&Action<KeyCode>, With<MainActionInput>>,
    mouse_button_actions: Query<&Action<MouseButton>, With<MainActionInput>>,
    touch_side_actions: Query<&Action<TouchSide>, With<MainActionInput>>,
    mut actions: Query<&mut Action<T>, With<W>>,
    action_maps: Query<&ActionMap<T>, With<W>>,
) where
    T: Default + Reflect + FromReflect + Debug + Send + Sync + Hash + Eq + Copy + Clone + 'static,
    W: Component,
{
    let keyboard_action = keyboard_actions.single();
    let mouse_button_action = mouse_button_actions.single();
    let touch_side_action = touch_side_actions.single();

    for mut action in actions.iter_mut() {
        let mut wants_on = HashSet::new();
        let mut wants_exit = HashSet::new();

        for action_map in action_maps.iter() {
            for action_map_input in action_map.iter() {
                let mut is_modified = true;
                if action_map_input.ctrl {
                    is_modified = is_modified && keyboard_action.on(KeyCode::LControl);
                } else {
                    is_modified = is_modified && !keyboard_action.on(KeyCode::LControl);
                }
                if action_map_input.shift {
                    is_modified = is_modified && keyboard_action.on(KeyCode::LShift);
                } else {
                    is_modified = is_modified && !keyboard_action.on(KeyCode::LShift);
                }
                if action_map_input.alt {
                    is_modified = is_modified && keyboard_action.on(KeyCode::LAlt);
                } else {
                    is_modified = is_modified && !keyboard_action.on(KeyCode::LAlt);
                }

                // Handle on_enter
                if is_modified {
                    match action_map_input.button {
                        ActionMapButton::Keyboard(key_code) => {
                            if keyboard_action.on_enter(key_code) {
                                if !action.on(action_map_input.action) {
                                    action.enter(action_map_input.action);
                                }
                            } else if keyboard_action.on(key_code) {
                                wants_on.insert(action_map_input.action);
                            }
                        }
                        ActionMapButton::MouseButton(mouse_button) => {
                            if mouse_button_action.on_enter(mouse_button) {
                                if !action.on(action_map_input.action) {
                                    action.enter(action_map_input.action);
                                }
                            } else if mouse_button_action.on(mouse_button) {
                                wants_on.insert(action_map_input.action);
                            }
                        }
                        ActionMapButton::TouchSide(touch_side) => {
                            if touch_side_action.on_enter(touch_side) {
                                if !action.on(action_map_input.action) {
                                    action.enter(action_map_input.action);
                                }
                            } else if touch_side_action.on(touch_side) {
                                wants_on.insert(action_map_input.action);
                            }
                        }
                        _ => panic!("Not implemented"),
                    }
                }

                // Handle on_exit
                match action_map_input.button {
                    ActionMapButton::Keyboard(key_code) => {
                        if keyboard_action.on_exit(key_code) && action.on(action_map_input.action) {
                            wants_exit.insert(action_map_input.action);
                        }
                    }
                    ActionMapButton::MouseButton(mouse_button) => {
                        if mouse_button_action.on_exit(mouse_button)
                            && action.on(action_map_input.action)
                        {
                            wants_exit.insert(action_map_input.action);
                        }
                    }
                    ActionMapButton::TouchSide(touch_side) => {
                        if touch_side_action.on_exit(touch_side)
                            && action.on(action_map_input.action)
                        {
                            wants_exit.insert(action_map_input.action);
                        }
                    }
                    _ => panic!("Not implemented"),
                }
            }
        }

        for input_action in wants_exit.iter() {
            if !wants_on.contains(input_action) {
                action.exit(*input_action);
            }
        }
    }
}

#[derive(Debug, Clone, Copy, Reflect, FromReflect, PartialEq, Eq, Hash)]
pub enum MouseAxis {
    X,
    Y,
    Z,
}

#[derive(Debug, Clone, Reflect, FromReflect)]
pub enum AxisMapSource {
    Keyboard {
        positive: KeyCode,
        negative: KeyCode,
    },
    MouseAxis(MouseAxis),
    GamepadAxis(Gamepad, GamepadAxis),
    TouchSideAxis(TouchSideAxis),
}

impl From<MouseAxis> for AxisMapSource {
    fn from(axis: MouseAxis) -> AxisMapSource {
        AxisMapSource::MouseAxis(axis)
    }
}

impl From<TouchSideAxis> for AxisMapSource {
    fn from(side_axis: TouchSideAxis) -> AxisMapSource {
        AxisMapSource::TouchSideAxis(side_axis)
    }
}

#[derive(Debug, Clone, Reflect, FromReflect)]
pub struct AxisMapInput<T>
where
    T: Reflect + Clone + Debug + Eq + Hash + Send + Sync + 'static,
{
    pub axis: T,
    pub source: AxisMapSource,
}

#[derive(Default, Debug, Deref, DerefMut, Component, Reflect, FromReflect)]
#[reflect(Component)]
pub struct ActionAxisMap<T>
where
    T: Default + Reflect + FromReflect + Debug + Send + Sync + Hash + Eq + Copy + Clone + 'static,
{
    map: Vec<AxisMapInput<T>>,
}

pub fn action_axis_map<T, W>(
    keyboard_actions: Query<&Action<KeyCode>, With<MainActionInput>>,
    mouse_axis_actions: Query<&ActionAxis<MouseAxis>, With<MainActionInput>>,
    touch_side_axis_actions: Query<&ActionAxis<TouchSideAxis>, With<MainActionInput>>,
    mut axes: Query<&mut ActionAxis<T>, With<W>>,
    axis_maps: Query<&ActionAxisMap<T>, With<W>>,
) where
    T: Default + Debug + Send + Sync + Hash + Eq + Copy + Clone + 'static + FromReflect,
    W: Component,
{
    let keyboard_action = keyboard_actions.single();
    let mouse_axis_action = mouse_axis_actions.single();
    let touch_side_axis_action = touch_side_axis_actions.single();

    for mut axis in axes.iter_mut() {
        axis.clear();
    }
    for mut axis in axes.iter_mut() {
        for axis_map in axis_maps.iter() {
            for axis_map_input in axis_map.map.iter() {
                let mut value = axis.get(axis_map_input.axis).unwrap_or(0.0);
                match axis_map_input.source {
                    AxisMapSource::Keyboard { positive, negative } => {
                        if keyboard_action.on(positive) {
                            value += 1.0;
                        }
                        if keyboard_action.on(negative) {
                            value -= 1.0;
                        }
                    }
                    AxisMapSource::MouseAxis(mouse_axis) => {
                        if let Some(mouse_value) = mouse_axis_action.get(mouse_axis) {
                            value += mouse_value;
                        }
                    }
                    AxisMapSource::TouchSideAxis(touch_side_axis) => {
                        if let Some(touch_side_value) = touch_side_axis_action.get(touch_side_axis)
                        {
                            value += touch_side_value;
                        }
                    }
                    _ => panic!("Not implemented"),
                }
                axis.set(axis_map_input.axis, value);
            }
        }
    }
}
