use bevy::{ecs::system::CommandQueue, prelude::*};
use common::*;
use simula_behavior::{spawn_tree, BehaviorCursor, BehaviorDocument, BehaviorTrace};

mod common;

#[test]
fn repeater_debug_succeed() {
    // Load behavior tree from RON string
    let data_str = r#"
        (root:(
            Repeater((repeat:Times(2), repeated:0)),
            [(Sequence(()),[
                (DebugAction((message:"Hello, from DebugMessage0!", fail:false, repeat:0)),[]),
                (DebugAction((message:"Hello, from DebugMessage1!", fail:false, repeat:2)),[])
            ])]
        ))
    "#;
    let document = ron::from_str::<BehaviorDocument<TestBehavior>>(data_str);
    assert!(document.is_ok());
    let document = document.unwrap();

    // Create app
    let mut app = App::new();
    test_app(&mut app);
    let mut command_queue = CommandQueue::default();
    let mut commands = Commands::new(&mut command_queue, &app.world);

    // Spawn tree
    let entity = spawn_tree(None, &mut commands, &document.root);
    commands.entity(entity).insert(BehaviorCursor);

    // Apply commands
    command_queue.apply(&mut app.world);

    // Run app
    let mut iters = 0;
    while iters < MAX_ITERS {
        iters = iters + 1;
        app.update();
    }

    // Confirm behavior trace
    let trace = app.world.get_resource::<BehaviorTrace>().unwrap();
    // println!("{:#?}", trace);
    let expected_trace = BehaviorTrace::from_list(&[
        "[0] + simula_behavior::decorators::repeater::Repeater is ready, starting",
        "[1] + simula_behavior::composites::sequence::Sequence is ready, starting",
        "[2] + simula_behavior::actions::debug_action::DebugAction is ready, starting",
        "[2] - simula_behavior::actions::debug_action::DebugAction is done, completing",
        "[3] + simula_behavior::actions::debug_action::DebugAction is ready, starting",
        "[3] - simula_behavior::actions::debug_action::DebugAction is done, completing",
        "[1] - simula_behavior::composites::sequence::Sequence is done, completing",
        "[0] + simula_behavior::decorators::repeater::Repeater is ready, starting",
        "[1] + simula_behavior::composites::sequence::Sequence is ready, starting",
        "[2] + simula_behavior::actions::debug_action::DebugAction is ready, starting",
        "[2] - simula_behavior::actions::debug_action::DebugAction is done, completing",
        "[3] + simula_behavior::actions::debug_action::DebugAction is ready, starting",
        "[3] - simula_behavior::actions::debug_action::DebugAction is done, completing",
        "[1] - simula_behavior::composites::sequence::Sequence is done, completing",
        "[0] - simula_behavior::decorators::repeater::Repeater is done, completing",
    ]);
    assert_eq!(trace, &expected_trace);
}
