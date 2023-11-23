use crate::{clear_behavior_started, complete_behavior, prelude::*, start_behavior, BehaviorTrace};
use bevy::{
    ecs::system::{CommandQueue, EntityCommands},
    prelude::*,
    reflect::TypeUuid,
};
use serde::{Deserialize, Serialize};
use simula_behavior_macro::BehaviorFactory;
use simula_script::{Script, ScriptContext};

pub const MAX_ITERS: usize = 200;

pub fn test_app(app: &mut App) -> &mut App {
    app.add_plugins(AssetPlugin::default());
    app.init_asset::<Script>();
    app.init_asset::<ScriptContext>();
    // Add the behaviors system to the app
    app.add_systems(
        Update,
        (clear_behavior_started, complete_behavior, start_behavior).chain(),
    );
    app.add_systems(
        Update,
        (
            debug::run,
            selector::run,
            sequencer::run,
            all::run,
            any::run,
            repeater::run,
            inverter::run,
            succeeder::run,
            wait::run,
            delay::run,
            identity::run,
            guard::run,
        ),
    );
    app.init_resource::<BehaviorTrace>();
    app
}

#[derive(Default, Debug, Serialize, Deserialize, Clone)]
pub struct TestBehaviorAttributes;

#[derive(Serialize, Deserialize, TypeUuid, Debug, Clone, Reflect, BehaviorFactory)]
#[uuid = "3d6cc56a-542e-11ed-9abb-02a179e5df2b"]
#[BehaviorAttributes(TestBehaviorAttributes)]
pub enum TestBehavior {
    Debug(Debug),
    Selector(Selector),
    Sequencer(Sequencer),
    All(All),
    Any(Any),
    Repeater(Repeater),
    Inverter(Inverter),
    Succeeder(Succeeder),
    Wait(Wait),
    Delay(Delay),
    Identity(Identity),
    Guard(Guard),
    Timeout(Timeout),
}

impl Default for TestBehavior {
    fn default() -> Self {
        Self::Debug(Debug::default())
    }
}

pub fn trace_behavior(behavior: &str) -> BehaviorTrace {
    // Load behavior tree from RON string
    let document = ron::from_str::<Behavior<TestBehavior>>(behavior);
    assert!(document.is_ok());
    let document = document.unwrap();
    // println!("Loaded behavior tree: \n{:#?}", ron::ser::to_string_pretty(&document, Default::default()).unwrap());

    // Create app
    let mut app = App::new();
    app.add_plugins(bevy::time::TimePlugin::default());
    test_app(&mut app);
    let mut command_queue = CommandQueue::default();
    let mut commands = Commands::new(&mut command_queue, &app.world);

    // Spawn tree
    let entity = commands.spawn_empty().id();
    let root = BehaviorTree::insert_tree(entity, None, &mut commands, &document);
    commands.entity(entity).add_child(root);
    commands.entity(root).insert(BehaviorCursor::Delegate);

    // Apply commands
    command_queue.apply(&mut app.world);

    // Run app
    let mut iters = 0;
    while iters < MAX_ITERS {
        iters += 1;
        app.update();
    }

    // Get app trace
    app.world.get_resource::<BehaviorTrace>().unwrap().clone()
}
