use bevy::{ecs::system::CommandQueue, prelude::*};
use simula_behavior::{
    actions::*, add_children, complete_behavior, composites::*, BehaviorCursor, BehaviorFailure,
    BehaviorInfo, BehaviorRunning, BehaviorSuccess, BehaviorTree,
};

fn test_app(app: &mut App) -> &mut App {
    // Add the behaviors system to the app
    app.add_system(complete_behavior);
    app.add_system(sequence::run);
    app.add_system(selector::run);
    app.add_system(debug_action::run);

    app
}

#[test]
fn root_success() {
    let mut app = App::new();

    test_app(&mut app);

    let mut command_queue = CommandQueue::default();
    let mut commands = Commands::new(&mut command_queue, &app.world);

    let mut root_node = commands.spawn();
    Sequence::spawn(&mut root_node);
    let root_node = root_node
        .insert(BehaviorRunning)
        .insert(BehaviorCursor)
        .id();

    commands
        .spawn()
        .insert(BehaviorTree {
            root: Some(root_node),
        })
        .push_children(&[root_node]);

    command_queue.apply(&mut app.world);

    app.update();

    let state = app.world.get::<BehaviorSuccess>(root_node);
    assert!(state.is_some());
}

#[test]
fn sequence_debug_success() {
    let mut app = App::new();

    test_app(&mut app);

    let mut command_queue = CommandQueue::default();
    let mut commands = Commands::new(&mut command_queue, &app.world);

    let mut root_node = commands.spawn();
    Sequence::spawn(&mut root_node);
    let root_node = root_node
        .insert(BehaviorRunning)
        .insert(BehaviorCursor)
        .id();

    let mut debug_message_0 = commands.spawn();
    DebugAction::spawn(&mut debug_message_0);
    debug_message_0.insert(DebugAction {
        message: "Hello, from DebugMessage 0!".to_string(),
        fail: false,
        repeat: 1,
    });
    debug_message_0.insert(Name::new("DebugMessage0"));
    let debug_message_0 = debug_message_0.id();

    let mut debug_message_1 = commands.spawn();
    DebugAction::spawn(&mut debug_message_1);
    debug_message_1.insert(DebugAction {
        message: "Hello, from DebugMessage 1!".to_string(),
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
    let mut max_iters = 0;
    while app.world.get::<BehaviorRunning>(debug_message_0).is_none() {
        max_iters = max_iters + 1;
        assert!(max_iters < 5);
        app.update();
    }

    // Confirm debug_message_0 is only behavior with cursor
    let mut debug_message_0_query = app.world.query_filtered::<Entity, With<BehaviorCursor>>();
    let entity = debug_message_0_query.single(&app.world);
    assert_eq!(debug_message_0, entity);

    // Run until debug_message_1 is Running
    let mut max_iters = 0;
    while app.world.get::<BehaviorRunning>(debug_message_1).is_none() {
        max_iters = max_iters + 1;
        assert!(max_iters < 5);
        app.update();
    }

    // Confirm debug_message_1 is only behavior with cursor
    let mut debug_message_1_query = app.world.query_filtered::<Entity, With<BehaviorCursor>>();
    let entity = debug_message_1_query.single(&app.world);
    assert_eq!(debug_message_1, entity);

    // Run until root is Success
    let mut max_iters = 0;
    while app.world.get::<BehaviorSuccess>(root_node).is_none() {
        max_iters = max_iters + 1;
        assert!(max_iters < 5);
        app.update();
    }

    // Give app chance to complete behavior
    app.update();

    // Confirm there are no cursors
    let mut cursors = app.world.query::<&BehaviorCursor>();
    assert_eq!(cursors.iter(&app.world).count(), 0);
}

#[test]
fn sequence_debug_failure() {
    let mut app = App::new();

    test_app(&mut app);

    let mut command_queue = CommandQueue::default();
    let mut commands = Commands::new(&mut command_queue, &app.world);

    let mut root_node = commands.spawn();
    Sequence::spawn(&mut root_node);
    let root_node = root_node
        .insert(BehaviorRunning)
        .insert(BehaviorCursor)
        .id();

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
    let mut max_iters = 0;
    while app.world.get::<BehaviorRunning>(debug_message_0).is_none() {
        max_iters = max_iters + 1;
        assert!(max_iters < 5);
        app.update();
    }

    // Confirm debug_message_0 is only behavior with cursor
    let mut debug_message_0_query = app.world.query_filtered::<Entity, With<BehaviorCursor>>();
    let entity = debug_message_0_query.single(&app.world);
    assert_eq!(debug_message_0, entity);

    // Run until debug_message_1 is Running
    let mut max_iters = 0;
    while app.world.get::<BehaviorRunning>(debug_message_1).is_none() {
        max_iters = max_iters + 1;
        assert!(max_iters < 5);
        app.update();
    }

    // Confirm debug_message_1 is only behavior with cursor
    let mut debug_message_1_query = app.world.query_filtered::<Entity, With<BehaviorCursor>>();
    let entity = debug_message_1_query.single(&app.world);
    assert_eq!(debug_message_1, entity);

    // Run until root is Failure
    let mut max_iters = 0;
    while app.world.get::<BehaviorFailure>(root_node).is_none() {
        max_iters = max_iters + 1;
        assert!(max_iters < 5);
        app.update();
    }

    // Give app chance to complete behavior
    app.update();

    // Confirm there are no cursors
    let mut cursors = app.world.query::<&BehaviorCursor>();
    assert_eq!(cursors.iter(&app.world).count(), 0);
}

#[test]
fn selector_debug_success() {
    let mut app = App::new();

    test_app(&mut app);

    let mut command_queue = CommandQueue::default();
    let mut commands = Commands::new(&mut command_queue, &app.world);

    let mut root_node = commands.spawn();
    Selector::spawn(&mut root_node);
    let root_node = root_node
        .insert(BehaviorRunning)
        .insert(BehaviorCursor)
        .id();

    let mut debug_message_0 = commands.spawn();
    DebugAction::spawn(&mut debug_message_0);
    debug_message_0.insert(DebugAction {
        message: "Hello, from DebugMessage 0!".to_string(),
        fail: false,
        repeat: 0,
    });
    debug_message_0.insert(Name::new("DebugMessage0"));
    let debug_message_0 = debug_message_0.id();

    let mut debug_message_1 = commands.spawn();
    DebugAction::spawn(&mut debug_message_1);
    debug_message_1.insert(DebugAction {
        message: "Hello, from DebugMessage 1!".to_string(),
        fail: false,
        repeat: 0,
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
    let mut max_iters = 0;
    while app.world.get::<BehaviorRunning>(debug_message_0).is_none() {
        max_iters = max_iters + 1;
        assert!(max_iters < 5);
        app.update();
    }

    // Confirm debug_message_0 is only behavior with cursor
    let mut debug_message_0_query = app.world.query_filtered::<Entity, With<BehaviorCursor>>();
    let entity = debug_message_0_query.single(&app.world);
    assert_eq!(debug_message_0, entity);

    // Run until root is Success
    let mut max_iters = 0;
    while app.world.get::<BehaviorSuccess>(root_node).is_none() {
        max_iters = max_iters + 1;
        assert!(max_iters < 5);
        app.update();
    }

    // Give app chance to complete behavior
    app.update();

    // Confirm there are no cursors
    let mut cursors = app.world.query::<&BehaviorCursor>();
    assert_eq!(cursors.iter(&app.world).count(), 0);

    // Confirm debug_message_0 is not Running, is Success, not Failure
    assert!(app.world.get::<BehaviorRunning>(debug_message_0).is_none());
    assert!(app.world.get::<BehaviorSuccess>(debug_message_0).is_some());
    assert!(app.world.get::<BehaviorFailure>(debug_message_0).is_none());

    // Confirm debug_message_1 is not Running, not Success, not Failure
    assert!(app.world.get::<BehaviorRunning>(debug_message_1).is_none());
    assert!(app.world.get::<BehaviorSuccess>(debug_message_1).is_none());
    assert!(app.world.get::<BehaviorFailure>(debug_message_1).is_none());
}

#[test]
fn selector_debug_failure() {
    let mut app = App::new();

    test_app(&mut app);

    let mut command_queue = CommandQueue::default();
    let mut commands = Commands::new(&mut command_queue, &app.world);

    let mut root_node = commands.spawn();
    Selector::spawn(&mut root_node);
    let root_node = root_node
        .insert(BehaviorRunning)
        .insert(BehaviorCursor)
        .id();

    let mut debug_message_0 = commands.spawn();
    DebugAction::spawn(&mut debug_message_0);
    debug_message_0.insert(DebugAction {
        message: "Hello, from DebugMessage 0!".to_string(),
        fail: true,
        repeat: 0,
    });
    debug_message_0.insert(Name::new("DebugMessage0"));
    let debug_message_0 = debug_message_0.id();

    let mut debug_message_1 = commands.spawn();
    DebugAction::spawn(&mut debug_message_1);
    debug_message_1.insert(DebugAction {
        message: "Hello, from DebugMessage 1!".to_string(),
        fail: true,
        repeat: 0,
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
    let mut max_iters = 0;
    while app.world.get::<BehaviorRunning>(debug_message_0).is_none() {
        max_iters = max_iters + 1;
        assert!(max_iters < 5);
        app.update();
    }

    // Confirm debug_message_0 is only behavior with cursor
    let mut debug_message_0_query = app.world.query_filtered::<Entity, With<BehaviorCursor>>();
    let entity = debug_message_0_query.single(&app.world);
    assert_eq!(debug_message_0, entity);

    // Run until debug_message_1 is Running
    let mut max_iters = 0;
    while app.world.get::<BehaviorRunning>(debug_message_1).is_none() {
        max_iters = max_iters + 1;
        assert!(max_iters < 5);
        app.update();
    }

    // Confirm debug_message_1 is only behavior with cursor
    let mut debug_message_1_query = app.world.query_filtered::<Entity, With<BehaviorCursor>>();
    let entity = debug_message_1_query.single(&app.world);
    assert_eq!(debug_message_1, entity);

    // Run until root is Failure
    let mut max_iters = 0;
    while app.world.get::<BehaviorFailure>(root_node).is_none() {
        max_iters = max_iters + 1;
        assert!(max_iters < 5);
        app.update();
    }

    // Give app chance to complete behavior
    app.update();

    // Confirm there are no cursors
    let mut cursors = app.world.query::<&BehaviorCursor>();
    assert_eq!(cursors.iter(&app.world).count(), 0);

    // Confirm debug_message_0 is not Running, not Success, is Failure
    assert!(app.world.get::<BehaviorRunning>(debug_message_0).is_none());
    assert!(app.world.get::<BehaviorSuccess>(debug_message_0).is_none());
    assert!(app.world.get::<BehaviorFailure>(debug_message_0).is_some());

    // Confirm debug_message_1 is not Running, not Success, is Failure
    assert!(app.world.get::<BehaviorRunning>(debug_message_1).is_none());
    assert!(app.world.get::<BehaviorSuccess>(debug_message_1).is_none());
    assert!(app.world.get::<BehaviorFailure>(debug_message_1).is_some());
}
