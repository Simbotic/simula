use bevy::input::{keyboard::KeyCode, mouse::MouseButton};
use bevy::{prelude::*, reflect::FromReflect};
use simula_action::{
    action_axis_map, action_map, Action, ActionAxis, ActionAxisMap, ActionMap, ActionMapInput,
    AxisMapInput, AxisMapSource, MouseAxis,
};

#[derive(Debug, PartialEq, Eq, Clone, Hash, SystemSet)]
pub struct MonkeyPlugin;

impl Plugin for MonkeyPlugin {
    fn build(&self, app: &mut App) {
        app.register_type::<MonkeyActor>()
            .register_type::<Action<MonkeyDo>>()
            .register_type::<Action<MonkeyMove>>()
            .register_type::<ActionMap<MonkeyDo>>()
            .register_type::<ActionAxisMap<MonkeyMove>>()
            .add_startup_system(monkey_setup)
            .add_system(action_map::<MonkeyDo, MonkeyActor>)
            .add_system(action_axis_map::<MonkeyMove, MonkeyActor>)
            .add_system(action_axis_map::<MonkeyLook, MonkeyActor>)
            .add_system(monkey_play);
    }
}

// Marker for entity controlled by these actions
#[derive(Component, Reflect)]
pub struct MonkeyActor;

// Actions
#[derive(Debug, Default, Hash, PartialEq, Eq, Clone, Copy, Reflect, FromReflect)]
pub enum MonkeyDo {
    #[default]
    Idle,
    Jump,
    Push,
    Hack,
}

// Move action axes
#[derive(Debug, Default, Hash, PartialEq, Eq, Clone, Copy, Reflect, FromReflect)]
pub enum MonkeyMove {
    #[default]
    Idle,
    Front,
    Right,
    Strafe,
    Zoom,
}

// Look around action axes
#[derive(Debug, Default, Hash, PartialEq, Eq, Clone, Copy, Reflect, FromReflect)]
pub enum MonkeyLook {
    #[default]
    Idle,
    Right,
    Up,
}

fn monkey_setup(mut commands: Commands) {
    let mut do_action_map: ActionMap<MonkeyDo> = Default::default();
    *do_action_map = vec![
        ActionMapInput {
            action: MonkeyDo::Jump,
            button: MouseButton::Left.into(),
            ctrl: false,
            shift: false,
            alt: false,
        },
        ActionMapInput {
            action: MonkeyDo::Jump,
            button: KeyCode::Space.into(),
            ctrl: false,
            shift: false,
            alt: false,
        },
        ActionMapInput {
            action: MonkeyDo::Push,
            button: KeyCode::A.into(),
            ctrl: true,
            shift: false,
            alt: false,
        },
        ActionMapInput {
            action: MonkeyDo::Push,
            button: MouseButton::Left.into(),
            ctrl: true,
            shift: false,
            alt: false,
        },
        ActionMapInput {
            action: MonkeyDo::Hack,
            button: KeyCode::H.into(),
            ctrl: false,
            shift: false,
            alt: false,
        },
    ];

    let mut move_axis_map: ActionAxisMap<MonkeyMove> = Default::default();
    *move_axis_map = vec![
        // WASD
        AxisMapInput {
            axis: MonkeyMove::Front,
            source: AxisMapSource::Keyboard {
                positive: KeyCode::W,
                negative: KeyCode::S,
            },
        },
        AxisMapInput {
            axis: MonkeyMove::Right,
            source: AxisMapSource::Keyboard {
                positive: KeyCode::D,
                negative: KeyCode::A,
            },
        },
        // Arrows
        AxisMapInput {
            axis: MonkeyMove::Strafe,
            source: AxisMapSource::Keyboard {
                positive: KeyCode::Right,
                negative: KeyCode::Left,
            },
        },
        // Mouse wheel
        AxisMapInput {
            axis: MonkeyMove::Zoom,
            source: AxisMapSource::MouseAxis(MouseAxis::Z),
        },
    ];

    let mut look_axis_map: ActionAxisMap<MonkeyLook> = Default::default();
    *look_axis_map = vec![
        // Mouse
        AxisMapInput {
            axis: MonkeyLook::Right,
            source: AxisMapSource::MouseAxis(MouseAxis::X),
        },
        AxisMapInput {
            axis: MonkeyLook::Up,
            source: AxisMapSource::MouseAxis(MouseAxis::Y),
        },
    ];

    commands
        .spawn_empty()
        .insert(MonkeyActor)
        .insert(Action::<MonkeyDo>::default())
        .insert(ActionAxis::<MonkeyMove>::default())
        .insert(ActionAxis::<MonkeyLook>::default())
        .insert(do_action_map)
        .insert(move_axis_map)
        .insert(look_axis_map)
        .insert(Name::new("Actor: Monkey"));
}

fn monkey_play(
    mut do_actions: Query<&mut Action<MonkeyDo>>,
    mut move_axes: Query<&mut ActionAxis<MonkeyMove>>,
    mut look_axes: Query<&mut ActionAxis<MonkeyLook>>,
) {
    let mut do_action = do_actions.single_mut();
    let move_axis = move_axes.single_mut();
    let look_axis = look_axes.single_mut();
    debug!("{:?}", do_action);
    debug!("{:?} {:?}", move_axis, look_axis);
    do_action.clear();
}
