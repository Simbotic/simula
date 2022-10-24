use bevy::prelude::*;
use simula_behavior::{actions::*, add_children, composites::*, BehaviorInfo, BehaviorTree, BehaviorCursor};

pub mod agent_rest;
pub mod agent_work;

pub struct AgentBehaviorPlugin;

impl Plugin for AgentBehaviorPlugin {
    fn build(&self, app: &mut App) {
        app
        // .add_system(agent_rest::run)
        // .add_system(agent_work::run)
        .add_system(test);
    }
}

pub fn test(query: Query<&BehaviorCursor>) {
    let cursors_count = query.iter().count();
    if cursors_count > 1 {
        panic!("There should only be one behavior cursor, but there are {}!", cursors_count);
    }
}

pub fn create(commands: &mut Commands) {
    let mut root_node = commands.spawn();
    Sequence::spawn(&mut root_node);
    let root_node = root_node.id();

    let mut debug_message_0 = commands.spawn();
    DebugMessage::spawn(&mut debug_message_0);
    debug_message_0.insert(DebugMessage {
        message: "Hello, from DebugMessage 0!".to_string(),
    });
    debug_message_0.insert(Name::new("DebugMessage0"));
    let debug_message_0 = debug_message_0.id();

    let mut debug_message_1 = commands.spawn();
    DebugMessage::spawn(&mut debug_message_1);
    debug_message_1.insert(DebugMessage {
        message: "Hello, from DebugMessage 1!".to_string(),
    });
    debug_message_1.insert(Name::new("DebugMessage1"));
    let debug_message_1 = debug_message_1.id();

    add_children(commands, root_node, &[debug_message_0, debug_message_1]);

    commands
        .spawn()
        .insert(BehaviorTree {
            root: Some(root_node),
        })
        .push_children(&[root_node]);
}
