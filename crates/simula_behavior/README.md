# Behavior System for AI, Decisions, State Management and Flow Control

## Create Behavior types

### Behavior Node types
Create enum with all possible behavior nodes to support.

```
#[derive(Serialize, Deserialize, TypeUuid)]
#[uuid = "5c3fbd4c-5359-11ed-9c5d-02a179e5df2b"]
pub enum MyBehavior {
    DebugAction(DebugAction),
    Selector(Selector),
    Sequence(Sequence),
}

impl Default for MyBehavior {
    fn default() -> Self {
        Self::DebugAction(DebugAction::default())
    }
}
```

### impl BehaviorSpawner
Let the behavior system know how to spawn behavior nodes. Usually the following example is enough.

```
impl BehaviorSpawner for MyBehavior {
    fn spawn_with(&self, commands: &mut EntityCommands) {
        match self {
            MyBehavior::DebugAction(action) => BehaviorInfo::spawn_with(commands, action),
            MyBehavior::Selector(selector) => BehaviorInfo::spawn_with(commands, selector),
            MyBehavior::Sequence(sequence) => BehaviorInfo::spawn_with(commands, sequence),
        }
    }
}
```
