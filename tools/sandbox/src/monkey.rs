use bevy::input::{keyboard::KeyCode, mouse::MouseButton};
use bevy::prelude::*;
use simula_action::{action_map, Action, ActionMap, ActionMapInput};

#[derive(Debug, PartialEq, Eq, Clone, Hash, SystemLabel)]
pub struct MonkeyPlugin;

impl Plugin for MonkeyPlugin {
    fn build(&self, app: &mut App) {
        app.add_startup_system(monkey_setup)
            .add_system(action_map::<MonkeyDo>)
            .add_system(monkey_play);
    }
}

#[derive(Component)]
pub struct MonkeyActor;

#[derive(Debug, Hash, PartialEq, Eq, Clone, Copy)]
pub enum MonkeyDo {
    Jump,
    Push,
    Hack,
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

    commands
        .spawn()
        .insert(MonkeyActor)
        .insert(Action::<MonkeyDo>::default())
        .insert(action_map);
}

fn monkey_play(mut actions: Query<&mut Action<MonkeyDo>>) {
    let mut action = actions.single_mut();
    println!("{:?}", action);
    action.clear();
}
