use bevy::{ecs::system::CommandQueue, prelude::*};
use simula_behavior::{
    composites::*, test::*, BehaviorCursor, BehaviorDocument, BehaviorFailure, BehaviorInfo, actions::*, add_children,
    BehaviorRunning, BehaviorSuccess, BehaviorTree,
};

#[test]
fn root_primitive_success() {
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

#[test]
fn sequence_primitive_success() {
    let mut app = App::new();

    test_app(&mut app);

    let mut command_queue = CommandQueue::default();
    let mut commands = Commands::new(&mut command_queue, &app.world);

    let mut root_node = commands.spawn();
    Sequence::spawn(&mut root_node);
    let root_node = root_node.insert(BehaviorCursor).id();

    let mut debug_message_0 = commands.spawn();
    DebugAction::spawn(&mut debug_message_0);
    debug_message_0.insert(DebugAction {
        message: "Hello, from DebugMessage0!".to_string(),
        fail: false,
        repeat: 1,
    });
    debug_message_0.insert(Name::new("DebugMessage0"));
    let debug_message_0 = debug_message_0.id();

    let mut debug_message_1 = commands.spawn();
    DebugAction::spawn(&mut debug_message_1);
    debug_message_1.insert(DebugAction {
        message: "Hello, from DebugMessage1!".to_string(),
        fail: false,
        repeat: 1,
    });
    debug_message_1.insert(Name::new("DebugMessage1"));
    let debug_message_1 = debug_message_1.id();

    add_children(
        &mut commands,
        root_node,
        &[debug_message_0, debug_message_1],
    );

    commands
        .spawn()
        .insert(BehaviorTree {
            root: Some(root_node),
        })
        .push_children(&[root_node, debug_message_0, debug_message_1]);

    command_queue.apply(&mut app.world);

    // Confirm root is only behavior with cursor
    let mut root_query = app.world.query_filtered::<Entity, With<BehaviorCursor>>();
    let entity = root_query.single(&app.world);
    assert_eq!(root_node, entity);

    // Run until debug_message_0 is Running
    let mut iters = 0;
    while app.world.get::<BehaviorRunning>(debug_message_0).is_none() {
        iters = iters + 1;
        assert!(iters < MAX_ITERS);
        app.update();
    }

    // Confirm debug_message_0 is only behavior with cursor
    let mut debug_message_0_query = app.world.query_filtered::<Entity, With<BehaviorCursor>>();
    let entity = debug_message_0_query.single(&app.world);
    assert_eq!(debug_message_0, entity);

    // Run until debug_message_1 is Running
    let mut iters = 0;
    while app.world.get::<BehaviorRunning>(debug_message_1).is_none() {
        iters = iters + 1;
        assert!(iters < MAX_ITERS);
        app.update();
    }

    // Confirm debug_message_1 is only behavior with cursor
    let mut debug_message_1_query = app.world.query_filtered::<Entity, With<BehaviorCursor>>();
    let entity = debug_message_1_query.single(&app.world);
    assert_eq!(debug_message_1, entity);

    // Run until root is Success
    let mut iters = 0;
    while app.world.get::<BehaviorSuccess>(root_node).is_none() {
        iters = iters + 1;
        assert!(iters < MAX_ITERS);
        app.update();
    }

    // Give app chance to complete behavior
    app.update();

    // Confirm there are no cursors
    let mut cursors = app.world.query::<&BehaviorCursor>();
    assert_eq!(cursors.iter(&app.world).count(), 0);
}

#[test]
fn sequence_primitive_failure() {
    let mut app = App::new();

    test_app(&mut app);

    let mut command_queue = CommandQueue::default();
    let mut commands = Commands::new(&mut command_queue, &app.world);

    let mut root_node = commands.spawn();
    Sequence::spawn(&mut root_node);
    let root_node = root_node.insert(BehaviorCursor).id();

    let mut debug_message_0 = commands.spawn();
    DebugAction::spawn(&mut debug_message_0);
    debug_message_0.insert(DebugAction {
        message: "Hello, from DebugMessage0 should succeed!".to_string(),
        fail: false,
        repeat: 1,
    });
    debug_message_0.insert(Name::new("DebugMessage0"));
    let debug_message_0 = debug_message_0.id();

    let mut debug_message_1 = commands.spawn();
    DebugAction::spawn(&mut debug_message_1);
    debug_message_1.insert(DebugAction {
        message: "Hello, from DebugMessage1 that should fail!".to_string(),
        fail: true,
        repeat: 1,
    });
    debug_message_1.insert(Name::new("DebugMessage1"));
    let debug_message_1 = debug_message_1.id();

    add_children(
        &mut commands,
        root_node,
        &[debug_message_0, debug_message_1],
    );

    commands
        .spawn()
        .insert(BehaviorTree {
            root: Some(root_node),
        })
        .push_children(&[root_node, debug_message_0, debug_message_1]);

    command_queue.apply(&mut app.world);

    // Confirm root is only behavior with cursor
    let mut root_query = app.world.query_filtered::<Entity, With<BehaviorCursor>>();
    let entity = root_query.single(&app.world);
    assert_eq!(root_node, entity);

    // Run until debug_message_0 is Running
    let mut iters = 0;
    while app.world.get::<BehaviorRunning>(debug_message_0).is_none() {
        iters = iters + 1;
        assert!(iters < MAX_ITERS);
        app.update();
    }

    // Confirm debug_message_0 is only behavior with cursor
    let mut debug_message_0_query = app.world.query_filtered::<Entity, With<BehaviorCursor>>();
    let entity = debug_message_0_query.single(&app.world);
    assert_eq!(debug_message_0, entity);

    // Run until debug_message_1 is Running
    let mut iters = 0;
    while app.world.get::<BehaviorRunning>(debug_message_1).is_none() {
        iters = iters + 1;
        assert!(iters < MAX_ITERS);
        app.update();
    }

    // Confirm debug_message_1 is only behavior with cursor
    let mut debug_message_1_query = app.world.query_filtered::<Entity, With<BehaviorCursor>>();
    let entity = debug_message_1_query.single(&app.world);
    assert_eq!(debug_message_1, entity);

    // Run until root is Failure
    let mut iters = 0;
    while app.world.get::<BehaviorFailure>(root_node).is_none() {
        iters = iters + 1;
        assert!(iters < MAX_ITERS);
        app.update();
    }

    // Give app chance to complete behavior
    app.update();

    // Confirm there are no cursors
    let mut cursors = app.world.query::<&BehaviorCursor>();
    assert_eq!(cursors.iter(&app.world).count(), 0);
}
