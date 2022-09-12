use bevy::prelude::*;
use simula_input::{
    action::{Action, ActionInput, ActionState},
    input_channel_begin, input_channel_end,
    keyboard::InputKeyboard,
    mouse::InputMouseButton,
    InputChannel, InputChannelBegin, InputChannelEnd,
};

#[derive(Reflect, SystemLabel, Default, Debug, Hash)]
struct MonkeyHack;
#[derive(Reflect, SystemLabel, Default, Debug, Hash)]
struct MonkeyPush;
#[derive(Reflect, SystemLabel, Default, Debug, Hash)]
struct MonkeyJump;

// Create a mock app to test the input system
fn mock_app() -> App {
    let mut app = App::new();

    // Uncomment for logging during manual vscode tests
    app.add_plugin(bevy::log::LogPlugin::default());

    app.add_system(run);

    //
    app.add_system_to_stage(
        CoreStage::PreUpdate,
        input_channel_begin.label(InputChannelBegin),
    )
    .add_system_to_stage(
        CoreStage::PostUpdate,
        input_channel_end.label(InputChannelEnd),
    )
    .add_system_to_stage(
        CoreStage::PostUpdate,
        cleanup.label("test_cleanup").after(input_channel_end),
    );

    // Add keyboard and mouse input
    let keyboard_input = Input::<KeyCode>::default();
    app.insert_resource(keyboard_input);
    let mouse_button_input = Input::<MouseButton>::default();
    app.insert_resource(mouse_button_input);

    // Add input channels
    app.insert_resource(InputChannel {
        input: Input::<MouseButton>::default(),
        owner: None,
    });
    app.insert_resource(InputChannel {
        input: Input::<KeyCode>::default(),
        owner: None,
    });

    app
}

// Cleanup inputs after test
fn cleanup(mut key_code: ResMut<Input<KeyCode>>, mut mouse_button: ResMut<Input<MouseButton>>) {
    key_code.clear();
    mouse_button.clear();
}

// Get the input resource for Input<KeyCode>>
fn keyboard_input<'a>(app: &'a mut App) -> Mut<'a, Input<KeyCode>> {
    app.world.resource_mut::<Input<KeyCode>>()
}

// Get the input resource for Input<MouseButton>>
fn mouse_button_input<'a>(app: &'a mut App) -> Mut<'a, Input<MouseButton>> {
    app.world.resource_mut::<Input<MouseButton>>()
}

// Get the action resource for Action<MonkeyHack>
fn monkey_hack_action<'a>(app: &'a mut App) -> Mut<'a, Action<MonkeyHack>> {
    app.world.resource_mut::<Action<MonkeyHack>>()
}

// Get the action resource for Action<MonkeyPush>
fn monkey_push_action<'a>(app: &'a mut App) -> Mut<'a, Action<MonkeyPush>> {
    app.world.resource_mut::<Action<MonkeyPush>>()
}

//
fn monkey_jump_action<'a>(app: &'a mut App) -> Mut<'a, Action<MonkeyJump>> {
    app.world.resource_mut::<Action<MonkeyJump>>()
}

// Subscribe to the action resources for Action<...>
fn run(
    mut _monkey_hack_events: EventReader<Action<MonkeyHack>>,
    mut _monkey_push_events: EventReader<Action<MonkeyPush>>,
    mut _monkey_jump_events: EventReader<Action<MonkeyJump>>,
) {
}

#[test]
fn monkey_hack_action_basic() {
    let mut app = mock_app();

    Action::<MonkeyJump>::add(&mut app, &[], InputChannelBegin);
    Action::<MonkeyPush>::add(&mut app, &[], MonkeyJump);

    Action::<MonkeyHack>::add(
        &mut app,
        &[ActionInput::with_mouse_button(MouseButton::Left)],
        MonkeyPush,
    );

    keyboard_input(&mut app).press(KeyCode::Space);
    mouse_button_input(&mut app).press(MouseButton::Left);

    app.update();

    assert_eq!(monkey_hack_action(&mut app).state, ActionState::Begin);
    assert_eq!(monkey_push_action(&mut app).state, ActionState::Idle);
    assert_eq!(monkey_jump_action(&mut app).state, ActionState::Idle);

    app.update();

    assert_eq!(monkey_hack_action(&mut app).state, ActionState::InProgress);
    assert_eq!(monkey_push_action(&mut app).state, ActionState::Idle);
    assert_eq!(monkey_jump_action(&mut app).state, ActionState::Idle);

    mouse_button_input(&mut app).release(MouseButton::Left);
    app.update();

    assert_eq!(monkey_hack_action(&mut app).state, ActionState::End);
    assert_eq!(monkey_push_action(&mut app).state, ActionState::Idle);
    assert_eq!(monkey_jump_action(&mut app).state, ActionState::Idle);

    app.update();

    assert_eq!(monkey_hack_action(&mut app).state, ActionState::Idle);
    assert_eq!(monkey_push_action(&mut app).state, ActionState::Idle);
    assert_eq!(monkey_jump_action(&mut app).state, ActionState::Idle);
}

#[test]
fn monkey_push_shadow_action() {
    let mut app = mock_app();

    keyboard_input(&mut app).press(KeyCode::A);
    mouse_button_input(&mut app).press(MouseButton::Right);

    // MonkeyJump - should never be triggered, since it has no inputs
    Action::<MonkeyJump>::add(&mut app, &[], InputChannelBegin);

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

    app.update();

    assert_eq!(monkey_hack_action(&mut app).state, ActionState::Idle);
    assert_eq!(monkey_push_action(&mut app).state, ActionState::Begin);
    assert_eq!(monkey_jump_action(&mut app).state, ActionState::Idle);
}

#[test]
fn monkey_jump_action_many_inputs() {
    let mut app = mock_app();

    keyboard_input(&mut app).press(KeyCode::LControl);
    mouse_button_input(&mut app).press(MouseButton::Left);

    Action::<MonkeyJump>::add(
        &mut app,
        &[
            ActionInput::with_mouse_button(MouseButton::Right),
            ActionInput::with_mouse_button(MouseButton::Left),
        ],
        InputChannelBegin,
    );

    Action::<MonkeyHack>::add(&mut app, &[], MonkeyJump);
    Action::<MonkeyPush>::add(&mut app, &[], MonkeyHack);

    app.update();

    assert_eq!(monkey_hack_action(&mut app).state, ActionState::Idle);
    assert_eq!(monkey_push_action(&mut app).state, ActionState::Idle);
    assert_eq!(monkey_jump_action(&mut app).state, ActionState::Begin);

    app.update();

    assert_eq!(monkey_jump_action(&mut app).state, ActionState::InProgress);

    app.update();

    assert_eq!(monkey_jump_action(&mut app).state, ActionState::InProgress);
}

#[test]
fn monkey_modified_action_shadow() {
    let mut app = mock_app();

    keyboard_input(&mut app).press(KeyCode::LControl);
    keyboard_input(&mut app).press(KeyCode::Space);
    mouse_button_input(&mut app).press(MouseButton::Left);

    Action::<MonkeyJump>::add(
        &mut app,
        &[ActionInput::Keyboard(InputKeyboard {
            key_code: KeyCode::Space,
            ctrl: true,
            ..Default::default()
        })],
        InputChannelBegin,
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

    assert_eq!(monkey_hack_action(&mut app).state, ActionState::Idle);
    assert_eq!(monkey_push_action(&mut app).state, ActionState::Idle);
    assert_eq!(monkey_jump_action(&mut app).state, ActionState::Begin);
}

#[test]
fn monkey_modified_action_no_shadow() {
    let mut app = mock_app();

    keyboard_input(&mut app).press(KeyCode::Space);
    mouse_button_input(&mut app).press(MouseButton::Left);

    Action::<MonkeyJump>::add(
        &mut app,
        &[ActionInput::Keyboard(InputKeyboard {
            key_code: KeyCode::Space,
            ctrl: true,
            ..Default::default()
        })],
        InputChannelBegin,
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

    assert_eq!(monkey_hack_action(&mut app).state, ActionState::Begin);
    assert_eq!(monkey_push_action(&mut app).state, ActionState::Idle);
    assert_eq!(monkey_jump_action(&mut app).state, ActionState::Idle);
}

#[test]
fn monkey_action_keyboard_mouse() {
    let mut app = mock_app();

    keyboard_input(&mut app).press(KeyCode::LControl);
    mouse_button_input(&mut app).press(MouseButton::Left);

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
        InputChannelBegin,
    );

    Action::<MonkeyHack>::add(&mut app, &[], MonkeyJump);
    Action::<MonkeyPush>::add(&mut app, &[], MonkeyHack);

    app.add_system(run);

    app.update();

    assert_eq!(monkey_hack_action(&mut app).state, ActionState::Idle);
    assert_eq!(monkey_push_action(&mut app).state, ActionState::Idle);
    assert_eq!(monkey_jump_action(&mut app).state, ActionState::Begin);
}

#[test]
fn monkey_modified_action_keyboard_mouse_shadow() {
    let mut app = mock_app();

    keyboard_input(&mut app).press(KeyCode::LControl);
    mouse_button_input(&mut app).press(MouseButton::Left);

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
        InputChannelBegin,
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

    assert_eq!(monkey_hack_action(&mut app).state, ActionState::Idle);
    assert_eq!(monkey_push_action(&mut app).state, ActionState::Idle);
    assert_eq!(monkey_jump_action(&mut app).state, ActionState::Begin);
}

#[test]
fn monkey_modified_action_keyboard_mouse_no_shadow() {
    let mut app = mock_app();

    mouse_button_input(&mut app).press(MouseButton::Left);

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
        InputChannelBegin,
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

    assert_eq!(monkey_hack_action(&mut app).state, ActionState::Begin);
    assert_eq!(monkey_push_action(&mut app).state, ActionState::Idle);
    assert_eq!(monkey_jump_action(&mut app).state, ActionState::Idle);
}
