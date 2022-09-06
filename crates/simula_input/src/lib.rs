pub use action::{Action, ActionInput};
use bevy::{input::InputSystem, prelude::*};
pub use keyboard::InputKeyboard;
pub use mouse::InputMouseButton;

pub mod action;
mod egui;
pub mod keyboard;
pub mod mouse;

struct FlySplat;
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

            Action::<CookieClick>::add(
                app,
                &[ActionInput::KeyboardMouseButton(
                    InputKeyboard {
                        key_code: KeyCode::LControl,
                        ..Default::default()
                    },
                    InputMouseButton {
                        button: MouseButton::Left,
                    },
                )],
            );

            Action::<FlySplat>::add(
                app,
                &[
                    ActionInput::with_keycode(KeyCode::Space),
                    ActionInput::with_mouse_button(MouseButton::Left),
                ],
            );
    }
}

fn setup() {}

fn run(
    mut cookid_events: EventReader<Action<CookieClick>>,
    mut fly_events: EventReader<Action<FlySplat>>,
) {
    for evt in cookid_events.iter() {
        println!("CookieClick {:?}", evt.state);
    }
    for evt in fly_events.iter() {
        println!("FlySplat {:?}", evt.state);
    }
}

#[cfg(test)]
mod tests {

    use super::*;
    use action::ActionState;

    struct MonkeyHack;
    struct MonkeyPush;
    struct MonkeyJump;

    fn run(
        mut monkey_hack_events: EventReader<Action<MonkeyHack>>,
        mut monkey_push_events: EventReader<Action<MonkeyPush>>,
        mut monkey_jump_events: EventReader<Action<MonkeyJump>>,
    ) {
        for evt in monkey_hack_events.iter() {
            println!("monkey_hack_event {:?}", evt.state);
        }
        for evt in monkey_push_events.iter() {
            println!("monkey_push_event {:?}", evt.state);
        }
        for evt in monkey_jump_events.iter() {
            println!("monkey_jump_event {:?}", evt.state);
        }
    }

    fn actions(app: &mut App) -> &mut App {
        // MonkeyJump - KeyCode::Ctrl + MouseButton::Left
        Action::<MonkeyJump>::add(
            app,
            &[ActionInput::KeyboardMouseButton(
                InputKeyboard {
                    key_code: KeyCode::LControl,
                    ..Default::default()
                },
                InputMouseButton {
                    button: MouseButton::Left,
                },
            )],
        );

        // MonkeyHack - KeyCode::Space or MouseButton::Left
        Action::<MonkeyHack>::add(
            app,
            &[
                ActionInput::with_keycode(KeyCode::Space),
                ActionInput::with_mouse_button(MouseButton::Left),
            ],
        );

        // MonkeyPush - KeyCode::Space and MouseButton::Left
        Action::<MonkeyPush>::add(app, &[ActionInput::with_mouse_button(MouseButton::Right)]);

        app
    }

    #[test]
    fn monkey_hack_action() {
        let mut app = App::new();

        let mut keyboard_input = Input::<KeyCode>::default();
        keyboard_input.press(KeyCode::Space);
        app.insert_resource(keyboard_input);

        let mut mouse_button_input = Input::<MouseButton>::default();
        mouse_button_input.press(MouseButton::Left);
        app.insert_resource(mouse_button_input);

        actions(&mut app);

        app.add_system(run);

        app.update();

        let monkey_hack_action = app.world.resource::<Action<MonkeyHack>>();
        assert_eq!(monkey_hack_action.state, ActionState::Begin);

        let monkey_push_action = app.world.resource::<Action<MonkeyPush>>();
        assert_eq!(monkey_push_action.state, ActionState::Idle);

        let monkey_jump_action = app.world.resource::<Action<MonkeyJump>>();
        assert_eq!(monkey_jump_action.state, ActionState::Idle);
    }

    #[test]
    fn monkey_push_action() {
        let mut app = App::new();

        let mut keyboard_input = Input::<KeyCode>::default();
        keyboard_input.press(KeyCode::A);
        app.insert_resource(keyboard_input);

        let mut mouse_button_input = Input::<MouseButton>::default();
        mouse_button_input.press(MouseButton::Right);
        app.insert_resource(mouse_button_input);

        actions(&mut app);

        app.add_system(run);

        app.update();

        let monkey_hack_action = app.world.resource::<Action<MonkeyHack>>();
        assert_eq!(monkey_hack_action.state, ActionState::Idle);

        let monkey_push_action = app.world.resource::<Action<MonkeyPush>>();
        assert_eq!(monkey_push_action.state, ActionState::Begin);

        let monkey_jump_action = app.world.resource::<Action<MonkeyJump>>();
        assert_eq!(monkey_jump_action.state, ActionState::Idle);
    }

    #[test]
    fn monkey_jump_action() {
        let mut app = App::new();

        let mut keyboard_input = Input::<KeyCode>::default();
        keyboard_input.press(KeyCode::LControl);
        app.insert_resource(keyboard_input);

        let mouse_button_input = Input::<MouseButton>::default();
        // mouse_button_input.press(MouseButton::Left);
        app.insert_resource(mouse_button_input);

        actions(&mut app);

        app.add_system(run);

        app.update();

        // After the first update, the action is still idle because the mouse button is not pressed
        let mut mouse_button_input = app.world.resource_mut::<Input<MouseButton>>();
        mouse_button_input.press(MouseButton::Left);

        app.update();

        let monkey_hack_action = app.world.resource::<Action<MonkeyHack>>();
        assert_eq!(monkey_hack_action.state, ActionState::Idle);

        let monkey_push_action = app.world.resource::<Action<MonkeyPush>>();
        assert_eq!(monkey_push_action.state, ActionState::Idle);

        let monkey_jump_action = app.world.resource::<Action<MonkeyJump>>();
        assert_eq!(monkey_jump_action.state, ActionState::Begin);
    }
}
