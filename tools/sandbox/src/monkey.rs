use bevy::input::{keyboard::KeyCode, mouse::MouseButton};
use bevy::prelude::*;
use simula_action::{
    action_map, axis_map, Action, ActionAxis, ActionMap, ActionMapInput, AxisMap, AxisMapInput,
    AxisMapSource, MouseAxis,
};

#[derive(Debug, PartialEq, Eq, Clone, Hash, SystemLabel)]
pub struct MonkeyPlugin;

impl Plugin for MonkeyPlugin {
    fn build(&self, app: &mut App) {
        app.add_startup_system(monkey_setup)
            .add_system(action_map::<MonkeyDo, MonkeyActor>)
            .add_system(axis_map::<MonkeyMove, MonkeyActor>)
            .add_system(monkey_play);
    }
}

// Marker for entity controlled by these actions
#[derive(Component)]
pub struct MonkeyActor;

// Actions
#[derive(Debug, Hash, PartialEq, Eq, Clone, Copy)]
pub enum MonkeyDo {
    Jump,
    Push,
    Hack,
}

// Action axes
#[derive(Debug, Hash, PartialEq, Eq, Clone, Copy)]
pub enum MonkeyMove {
    Front,
    Right,
    Strafe,
    Zoom,
}

fn monkey_setup(mut commands: Commands) {
    let mut action_map: ActionMap<MonkeyDo> = Default::default();
    *action_map = vec![
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

    let mut axis_map: AxisMap<MonkeyMove> = Default::default();
    *axis_map = vec![
        // WASD
        AxisMapInput {
            axis: MonkeyMove::Front,
            source: AxisMapSource::Keyboard {
                positive: KeyCode::W.into(),
                negative: KeyCode::S.into(),
            },
        },
        AxisMapInput {
            axis: MonkeyMove::Right,
            source: AxisMapSource::Keyboard {
                positive: KeyCode::D.into(),
                negative: KeyCode::A.into(),
            },
        },
        // Arrows
        AxisMapInput {
            axis: MonkeyMove::Strafe,
            source: AxisMapSource::Keyboard {
                positive: KeyCode::Right.into(),
                negative: KeyCode::Left.into(),
            },
        },
        // Mouse wheel
        AxisMapInput {
            axis: MonkeyMove::Zoom,
            source: AxisMapSource::MouseAxis(MouseAxis::Z),
        },
    ];

    commands
        .spawn()
        .insert(MonkeyActor)
        .insert(Action::<MonkeyDo>::default())
        .insert(ActionAxis::<MonkeyMove>::default())
        .insert(action_map)
        .insert(axis_map)
        .insert(Name::new("Actor: Monkey"));
}

fn monkey_play(
    mut actions: Query<&mut Action<MonkeyDo>>,
    mut axes: Query<&mut ActionAxis<MonkeyMove>>,
) {
    let mut action = actions.single_mut();
    let axis = axes.single_mut();
    println!("{:?} {:?}", action, axis);
    action.clear();
}
