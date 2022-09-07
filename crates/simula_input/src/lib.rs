pub use action::{Action, ActionInput};
use bevy::{input::InputSystem, prelude::*};
pub use keyboard::InputKeyboard;
pub use mouse::InputMouseButton;

pub mod action;
mod egui;
pub mod keyboard;
pub mod mouse;

#[derive(Reflect, SystemLabel, Default, Debug)]
struct FlySplat;

#[derive(Reflect, SystemLabel, Default, Debug)]
struct CookieClick;

pub struct InputControlPlugin;

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
            .add_startup_system(setup)
            .add_system(run);

        // Action::<CookieClick>::add(
        //     app,
        //     &[
        //         ActionInput::with_keycode(KeyCode::Space),
        //         ActionInput::with_mouse_button(MouseButton::Left),
        //     ],
        //     InputSystem,
        // );

        // Action::<CookieClick>::add(
        //     app,
        //     &[ActionInput::Keyboard(InputKeyboard {
        //         key_code: KeyCode::Space,
        //         ctrl: true,
        //         ..Default::default()
        //     })],
        //     InputSystem,
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
            InputSystem,
        );

        Action::<FlySplat>::add(
            app,
            &[
                // ActionInput::with_keycode(KeyCode::Space),
                // ActionInput::with_mouse_button(MouseButton::Right),
                // ActionInput::with_mouse_button(MouseButton::Left),
            ],
            CookieClick,
        );
    }
}

fn setup() {}

fn run(
    mut cookid_events: EventReader<Action<CookieClick>>,
    mut fly_events: EventReader<Action<FlySplat>>,
) {
    // for evt in cookid_events.iter() {
    //     println!("CookieClick {:?}", evt.state);
    // }
    // for evt in fly_events.iter() {
    //     println!("FlySplat {:?}", evt.state);
    // }
}
