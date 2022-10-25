use bevy::{ecs::system::EntityCommands, prelude::*, reflect::TypeUuid};
use serde::{Deserialize, Serialize};
use simula_behavior::{
    actions::*, complete_behavior, composites::*, decorators::*, start_behavior, BehaviorInfo,
    BehaviorSpawner, BehaviorTrace,
};

pub const MAX_ITERS: usize = 200;

pub fn test_app(app: &mut App) -> &mut App {
    // Add the behaviors system to the app
    app.add_system(start_behavior);
    app.add_system(complete_behavior);
    app.add_system(sequence::run);
    app.add_system(selector::run);
    app.add_system(repeater::run);
    app.add_system(debug_action::run);
    app.init_resource::<BehaviorTrace>();
    app
}

#[derive(Serialize, Deserialize, TypeUuid)]
#[uuid = "3d6cc56a-542e-11ed-9abb-02a179e5df2b"]
pub enum TestBehavior {
    DebugAction(DebugAction),
    Selector(Selector),
    Sequence(Sequence),
    Repeater(Repeater),
}

impl Default for TestBehavior {
    fn default() -> Self {
        Self::DebugAction(DebugAction::default())
    }
}

impl BehaviorSpawner for TestBehavior {
    fn spawn_with(&self, commands: &mut EntityCommands) {
        match self {
            TestBehavior::DebugAction(action) => BehaviorInfo::spawn_with(commands, action),
            TestBehavior::Selector(selector) => BehaviorInfo::spawn_with(commands, selector),
            TestBehavior::Sequence(sequence) => BehaviorInfo::spawn_with(commands, sequence),
            TestBehavior::Repeater(repeater) => BehaviorInfo::spawn_with(commands, repeater),
        }
    }
}
