use bevy::input::{mouse::MouseButtonInput, InputSystem};
use bevy::prelude::*;
use simula_input::{
    action::{Action, ActionInput, ActionState},
    keyboard::InputKeyboard,
    mouse::InputMouseButton,
};

#[derive(Reflect, SystemLabel, Default, Debug)]
struct MonkeyHack;
#[derive(Reflect, SystemLabel, Default, Debug)]
struct MonkeyPush;
#[derive(Reflect, SystemLabel, Default, Debug)]
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

#[test]
fn monkey_hack_action() {
    let mut app = App::new();

    let mut keyboard_input = Input::<KeyCode>::default();
    keyboard_input.press(KeyCode::Space);
    app.insert_resource(keyboard_input);

    let mut mouse_button_input = Input::<MouseButton>::default();
    mouse_button_input.press(MouseButton::Left);
    app.insert_resource(mouse_button_input);

    Action::<MonkeyJump>::add(&mut app, &[], InputSystem);
    Action::<MonkeyPush>::add(&mut app, &[], InputSystem);

    Action::<MonkeyHack>::add(
        &mut app,
        &[ActionInput::with_mouse_button(MouseButton::Left)],
        MonkeyPush,
    );

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
fn monkey_push_shadow_action() {
    let mut app = App::new();

    let mut keyboard_input = Input::<KeyCode>::default();
    keyboard_input.press(KeyCode::A);
    app.insert_resource(keyboard_input);

    let mut mouse_button_input = Input::<MouseButton>::default();
    mouse_button_input.press(MouseButton::Right);
    app.insert_resource(mouse_button_input);

    // MonkeyJump - should never be triggered, since it has no inputs
    Action::<MonkeyJump>::add(&mut app, &[], InputSystem);

    // MonkeyPush - triggered by MouseButton::Right
    Action::<MonkeyPush>::add(
        &mut app,
        &[ActionInput::with_mouse_button(MouseButton::Right)],
        MonkeyJump,
    );

    // MonkeyHack - should be shadowed by MonkeyPush
    Action::<MonkeyHack>::add(
        &mut app,
        &[ActionInput::with_mouse_button(MouseButton::Right)],
        MonkeyPush,
    );

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
fn monkey_jump_action_many_inputs() {
    let mut app = App::new();

    let mut keyboard_input = Input::<KeyCode>::default();
    keyboard_input.press(KeyCode::LControl);
    app.insert_resource(keyboard_input);

    let mut mouse_button_input = Input::<MouseButton>::default();
    mouse_button_input.press(MouseButton::Left);
    app.insert_resource(mouse_button_input);

    Action::<MonkeyJump>::add(
        &mut app,
        &[
            ActionInput::with_mouse_button(MouseButton::Right),
            ActionInput::with_mouse_button(MouseButton::Left),
        ],
        InputSystem,
    );

    Action::<MonkeyHack>::add(&mut app, &[], MonkeyJump);
    Action::<MonkeyPush>::add(&mut app, &[], MonkeyHack);

    app.add_system(run);

    app.update();

    let monkey_hack_action = app.world.resource::<Action<MonkeyHack>>();
    assert_eq!(monkey_hack_action.state, ActionState::Idle);

    let monkey_push_action = app.world.resource::<Action<MonkeyPush>>();
    assert_eq!(monkey_push_action.state, ActionState::Idle);

    let monkey_jump_action = app.world.resource::<Action<MonkeyJump>>();
    assert_eq!(monkey_jump_action.state, ActionState::Begin);

    app.update();

    let monkey_jump_action = app.world.resource::<Action<MonkeyJump>>();
    assert_eq!(monkey_jump_action.state, ActionState::InProgress);

    app.update();

    let monkey_jump_action = app.world.resource::<Action<MonkeyJump>>();
    assert_eq!(monkey_jump_action.state, ActionState::InProgress);
}

#[test]
fn monkey_modified_action_shadow() {
    let mut app = App::new();

    let mut keyboard_input = Input::<KeyCode>::default();
    keyboard_input.press(KeyCode::LControl);
    keyboard_input.press(KeyCode::Space);
    app.insert_resource(keyboard_input);

    let mut mouse_button_input = Input::<MouseButton>::default();
    mouse_button_input.press(MouseButton::Left);
    app.insert_resource(mouse_button_input);

    Action::<MonkeyJump>::add(
        &mut app,
        &[ActionInput::Keyboard(InputKeyboard {
            key_code: KeyCode::Space,
            ctrl: true,
            ..Default::default()
        })],
        InputSystem,
    );

    // MonkeyHack - should not work if modifier from MonkeyJump is pressed
    Action::<MonkeyHack>::add(
        &mut app,
        &[ActionInput::Keyboard(InputKeyboard {
            key_code: KeyCode::Space,
            ..Default::default()
        })],
        MonkeyJump,
    );

    Action::<MonkeyPush>::add(&mut app, &[], MonkeyHack);

    app.add_system(run);

    app.update();

    let monkey_hack_action = app.world.resource::<Action<MonkeyHack>>();
    assert_eq!(monkey_hack_action.state, ActionState::Idle);

    let monkey_push_action = app.world.resource::<Action<MonkeyPush>>();
    assert_eq!(monkey_push_action.state, ActionState::Idle);

    let monkey_jump_action = app.world.resource::<Action<MonkeyJump>>();
    assert_eq!(monkey_jump_action.state, ActionState::Begin);
}

#[test]
fn monkey_modified_action_no_shadow() {
    let mut app = App::new();

    let mut keyboard_input = Input::<KeyCode>::default();
    // keyboard_input.press(KeyCode::LControl);
    keyboard_input.press(KeyCode::Space);
    app.insert_resource(keyboard_input);

    let mut mouse_button_input = Input::<MouseButton>::default();
    mouse_button_input.press(MouseButton::Left);
    app.insert_resource(mouse_button_input);

    Action::<MonkeyJump>::add(
        &mut app,
        &[ActionInput::Keyboard(InputKeyboard {
            key_code: KeyCode::Space,
            ctrl: true,
            ..Default::default()
        })],
        InputSystem,
    );

    // MonkeyHack - should work if modifier from MonkeyJump is not pressed
    Action::<MonkeyHack>::add(
        &mut app,
        &[ActionInput::Keyboard(InputKeyboard {
            key_code: KeyCode::Space,
            ..Default::default()
        })],
        MonkeyJump,
    );

    Action::<MonkeyPush>::add(&mut app, &[], MonkeyHack);

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
fn monkey_action_keyboard_mouse() {
    let mut app = App::new();

    let mut keyboard_input = Input::<KeyCode>::default();
    keyboard_input.press(KeyCode::LControl);
    app.insert_resource(keyboard_input);

    let mut mouse_button_input = Input::<MouseButton>::default();
    mouse_button_input.press(MouseButton::Left);
    app.insert_resource(mouse_button_input);

    Action::<MonkeyJump>::add(
        &mut app,
        &[ActionInput::KeyboardMouseButton(
            InputKeyboard {
                key_code: KeyCode::LControl,
                ..Default::default()
            },
            InputMouseButton {
                button: MouseButton::Left,
            },
        )],
        InputSystem,
    );

    Action::<MonkeyHack>::add(&mut app, &[], MonkeyJump);
    Action::<MonkeyPush>::add(&mut app, &[], MonkeyHack);

    app.add_system(run);

    app.update();

    let monkey_hack_action = app.world.resource::<Action<MonkeyHack>>();
    assert_eq!(monkey_hack_action.state, ActionState::Idle);

    let monkey_push_action = app.world.resource::<Action<MonkeyPush>>();
    assert_eq!(monkey_push_action.state, ActionState::Idle);

    let monkey_jump_action = app.world.resource::<Action<MonkeyJump>>();
    assert_eq!(monkey_jump_action.state, ActionState::Begin);
}

#[test]
fn monkey_modified_action_keyboard_mouse_shadow() {
    let mut app = App::new();

    let mut keyboard_input = Input::<KeyCode>::default();
    keyboard_input.press(KeyCode::LControl);
    app.insert_resource(keyboard_input);

    let mut mouse_button_input = Input::<MouseButton>::default();
    mouse_button_input.press(MouseButton::Left);
    app.insert_resource(mouse_button_input);

    Action::<MonkeyJump>::add(
        &mut app,
        &[ActionInput::KeyboardMouseButton(
            InputKeyboard {
                key_code: KeyCode::LControl,
                ..Default::default()
            },
            InputMouseButton {
                button: MouseButton::Left,
            },
        )],
        InputSystem,
    );

    // MonkeyHack - should not work if modifier from MonkeyJump is pressed
    Action::<MonkeyHack>::add(
        &mut app,
        &[ActionInput::MouseButton(InputMouseButton {
            button: MouseButton::Left,
        })],
        MonkeyJump,
    );

    Action::<MonkeyPush>::add(&mut app, &[], MonkeyHack);

    app.add_system(run);

    app.update();

    let monkey_hack_action = app.world.resource::<Action<MonkeyHack>>();
    assert_eq!(monkey_hack_action.state, ActionState::Idle);

    let monkey_push_action = app.world.resource::<Action<MonkeyPush>>();
    assert_eq!(monkey_push_action.state, ActionState::Idle);

    let monkey_jump_action = app.world.resource::<Action<MonkeyJump>>();
    assert_eq!(monkey_jump_action.state, ActionState::Begin);
}

#[test]
fn monkey_modified_action_keyboard_mouse_no_shadow() {
    let mut app = App::new();

    let keyboard_input = Input::<KeyCode>::default();
    app.insert_resource(keyboard_input);

    let mut mouse_button_input = Input::<MouseButton>::default();
    mouse_button_input.press(MouseButton::Left);
    app.insert_resource(mouse_button_input);

    Action::<MonkeyJump>::add(
        &mut app,
        &[ActionInput::KeyboardMouseButton(
            InputKeyboard {
                key_code: KeyCode::LControl,
                ..Default::default()
            },
            InputMouseButton {
                button: MouseButton::Left,
            },
        )],
        InputSystem,
    );

    // MonkeyHack - should work if modifier from MonkeyJump is not pressed
    Action::<MonkeyHack>::add(
        &mut app,
        &[ActionInput::MouseButton(InputMouseButton {
            button: MouseButton::Left,
        })],
        MonkeyJump,
    );

    Action::<MonkeyPush>::add(&mut app, &[], MonkeyHack);

    app.add_system(run);

    app.update();

    let monkey_hack_action = app.world.resource::<Action<MonkeyHack>>();
    assert_eq!(monkey_hack_action.state, ActionState::Begin);

    let monkey_push_action = app.world.resource::<Action<MonkeyPush>>();
    assert_eq!(monkey_push_action.state, ActionState::Idle);

    let monkey_jump_action = app.world.resource::<Action<MonkeyJump>>();
    assert_eq!(monkey_jump_action.state, ActionState::Idle);
}
