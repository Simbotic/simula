pub use action::{Action, ActionInput};
use bevy::{input::InputSystem, prelude::*};
pub use keyboard::InputKeyboard;
pub use mouse::InputMouseButton;
use std::hash::Hash;

pub mod action;
mod egui;
pub mod keyboard;
pub mod mouse;

#[derive(Reflect, SystemLabel, Default, Debug, Hash)]
struct FlySplat;

#[derive(Reflect, SystemLabel, Default, Debug, Hash)]
struct CookieClick;

pub struct InputControlPlugin;

#[derive(SystemLabel)]
pub struct InputChannelBegin;
#[derive(SystemLabel)]
pub struct InputChannelEnd;

pub struct InputChannel<T>
where
    T: Send + Sync + Hash + Eq + 'static,
{
    pub input: Input<T>,
    pub owner: Option<u64>,
}

impl Plugin for InputControlPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<egui::EguiBlockInputState>()
            .add_system_to_stage(
                CoreStage::PreUpdate,
                egui::egui_block_input.after(InputSystem),
            )
            .add_system_to_stage(
                CoreStage::PostUpdate,
                egui::egui_wants_input.after(egui::EguiSystem::ProcessOutput),
            )
            .add_system_to_stage(
                CoreStage::PreUpdate,
                input_channel_begin
                    .label(InputChannelBegin)
                    .after(InputSystem),
            )
            .add_system_to_stage(CoreStage::PostUpdate, input_channel_end.label(InputChannelEnd));
        // .add_startup_system(setup)
        // .add_system(run);

        app.insert_resource(InputChannel {
            input: Input::<MouseButton>::default(),
            owner: None,
        });

        app.insert_resource(InputChannel {
            input: Input::<KeyCode>::default(),
            owner: None,
        });

        // Action::<CookieClick>::add(
        //     app,
        //     &[
        //         ActionInput::with_keycode(KeyCode::Space),
        //         ActionInput::with_mouse_button(MouseButton::Left),
        //     ],
        //     InputChannelBegin,
        // );

        // Action::<CookieClick>::add(
        //     app,
        //     &[ActionInput::Keyboard(InputKeyboard {
        //         key_code: KeyCode::Space,
        //         ctrl: true,
        //         ..Default::default()
        //     })],
        //     InputChannelBegin,
        // );

        Action::<CookieClick>::add(
            app,
            &[
                ActionInput::KeyboardMouseButton(
                InputKeyboard {
                    key_code: KeyCode::LControl,
                    ..Default::default()
                },
                InputMouseButton {
                    button: MouseButton::Left,
                },
            )
            ],
            InputChannelBegin,
        );

        Action::<FlySplat>::add(
            app,
            &[
                ActionInput::with_keycode(KeyCode::Space),
                ActionInput::with_mouse_button(MouseButton::Right),
                ActionInput::with_mouse_button(MouseButton::Left),
            ],
            CookieClick,
        );

        // Action::<FlySplat>::add(
        //     app,
        //     &[ActionInput::Keyboard(InputKeyboard {
        //         key_code: KeyCode::P,
        //         ctrl: true,
        //         ..Default::default()
        //     })],
        //     InputChannelBegin,
        // );

    }
}

pub fn input_channel_begin(
    mut input_channel_mouse_button: ResMut<InputChannel<MouseButton>>,
    mut input_channel_keycode: ResMut<InputChannel<KeyCode>>,
    input_mouse_button: ResMut<Input<MouseButton>>,
    input_keycode: ResMut<Input<KeyCode>>,
) {
    input_channel_mouse_button.input = input_mouse_button.clone();
    input_channel_keycode.input = input_keycode.clone();
}

pub fn input_channel_end(
    mut _input_channel_mouse_button: ResMut<InputChannel<MouseButton>>,
    mut _input_channel_keycode: ResMut<InputChannel<KeyCode>>,
) {
}

// fn setup() {}

// fn run(
//     mut cookid_events: EventReader<Action<CookieClick>>,
//     mut fly_events: EventReader<Action<FlySplat>>,
// ) {
//     // for evt in cookid_events.iter() {
//     //     println!("CookieClick {:?}", evt.state);
//     // }
//     // for evt in fly_events.iter() {
//     //     println!("FlySplat {:?}", evt.state);
//     // }
// }
