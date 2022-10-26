use bevy::{ecs::system::CommandQueue, prelude::*};
use common::*;
use simula_behavior::{
    composites::*, BehaviorCursor, BehaviorDocument, BehaviorInfo, BehaviorSuccess, BehaviorTree,
};

mod common;

#[test]
fn root_success() {
    let mut app = App::new();

    test_app(&mut app);

    let mut command_queue = CommandQueue::default();
    let mut commands = Commands::new(&mut command_queue, &app.world);

    let mut root_node = commands.spawn();
    Sequence::spawn(&mut root_node);
    let root_node = root_node.insert(BehaviorCursor).id();

    commands
        .spawn()
        .insert(BehaviorTree {
            root: Some(root_node),
        })
        .push_children(&[root_node]);

    command_queue.apply(&mut app.world);

    let mut iters = 0;
    while app.world.get::<BehaviorSuccess>(root_node).is_some() {
        iters = iters + 1;
        assert!(iters < MAX_ITERS);
        app.update();
    }
}

#[test]
fn deserialize_behavior_tree() {
    let data_str = r#"
        (
            root:(
                Sequence(()),
                [
                    (DebugAction((message:"Hello, from DebugMessage0!", fail:false, repeat:0)), []),
                    (DebugAction((message:"Hello, from DebugMessage1!", fail:false, repeat:5)), []),
                ]
            )
        )
    "#;

    let data = ron::from_str::<BehaviorDocument<TestBehavior>>(data_str);
    assert!(data.is_ok());
}
