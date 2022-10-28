# Behavior System for AI, Decisions, State Management and Flow Control

## Create a Behavior System

- Behavior - All behavior nodes, custom actions, decorators and composites.
- BehaviorNode - One of many actions, decorators or composites.
- BehaviorDocument - A struct describing a configuraiton for an entire Behavior.
- BehaviorAsset - A bevy asset for integrating custom Behavior with asset system.

### Behavior
Create a Behavior enum with all behavior nodes to support. This enum will become part of the behavior's document asset. Make sure UUID is unique.

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
            MyBehavior::DebugAction(data) => BehaviorInfo::spawn_with(commands, data),
            MyBehavior::Selector(data) => BehaviorInfo::spawn_with(commands, data),
            MyBehavior::Sequence(data) => BehaviorInfo::spawn_with(commands, data),
        }
    }
}
```

### Behavior Plugin
Each behavior system should be contained in a plugin. The custom behavior async_loader and any new custom behavior nodes should be added here.

```
pub struct MyBehaviorPlugin;

impl Plugin for MyBehaviorPlugin {
    fn build(&self, app: &mut App) {
        app
            // Add your new behavior asset to the plugin
            .add_system(behavior_loader::<MyBehavior>)
            // Add any custom behavior nodes 
            .add_system(my_behavior::dummy_action::run)
            .add_system(my_behavior::dummy_flipper::run);
    }
}
```

### Creating Behaviors

From asset

```
let document: Handle<BehaviorAsset> = asset_server.load("my_behavior_test.bht.ron");
let behavior = BehaviorTree::from_asset::<MissionBehavior>(None, commands, &document);
```

From code

```
let document = BehaviorDocument {
    root: BTNode(
        MyBehavior::Repeater(Repeater {
            repeat: Repeat::Times(2),
            ..default()
        }),
        vec![BTNode(
            MyBehavior::Sequence(Sequence::default()),
            vec![
                BTNode(
                    MyBehavior::DebugAction(DebugAction {
                        message: "Hello, from DebugMessage0!".to_string(),
                        ..default()
                    }),
                    vec![],
                ),
                BTNode(
                    MyBehavior::DebugAction(DebugAction {
                        message: "Hello, from DebugMessage1!".to_string(),
                        repeat: 5,
                        ..default()
                    }),
                    vec![],
                ),
            ],
        )],
    ),
};
let behavior = BehaviorTree::from_document(None, commands, &document);
```

From text

```
let data_str = r#"
    (root:(
        Repeater((repeat:Times(2), repeated:0)),
        [(Sequence(()),[
            (DebugAction((message:"Hello, from DebugMessage0!", fail:false, repeat:0)),[]),
            (DebugAction((message:"Hello, from DebugMessage1!", fail:false, repeat:2)),[])
        ])]
    ))
"#;
let document = ron::from_str::<BehaviorDocument<MyBehavior>>(data_str);
let behavior = BehaviorTree::from_document(None, commands, &document);
```


## Instantiating and Start Behaviors

```
// Generate all entities with their behavior components
let behavior = BehaviorTree::from_document(None, commands, &document);

// Set cursor at root, so it starts running
commands.entity(behavior.root).insert(BehaviorCursor);

// New entity to organize behavior. Inserting the behavior component into a new entity and pushing the tree as child is optional, just for organization purposes.
commands.spawn().insert(behavior).push_children(&[behavior.root]);
```

## Custom Behavior Node
Create a Component as you would any other ECS component

```
#[derive(Debug, Default, Component, Reflect, Clone, Deserialize, Serialize)]
pub struct DebugAction {
    pub message: String,
    pub fail: bool,
    pub repeat: u8,
}
```

`impl BehaviorInfo` to aid in debug and tools.

```
impl BehaviorInfo for DebugAction {
    const TYPE: BehaviorType = BehaviorType::Action;
    const NAME: &'static str = "Debug Action";
    const DESC: &'static str = "Display a debug message and complete with success or failure";
}
```

Create a system as you would any other ECS system. Use the `BehaviorRunQuery` query filter to let the Behavior System take control. Once your action is done, insert a `BehaviorSuccess` component.

```
pub fn run(
    mut commands: Commands,
    mut debug_actions: Query<(Entity, &mut DebugAction), BehaviorRunQuery>,
) {
    for (entity, mut debug_action) in &mut debug_actions {
        debug!("[{}] {}", debug_action.repeat, debug_action.message);
        if debug_action.repeat == 0 {
            if debug_action.fail {
                commands.entity(entity).insert(BehaviorFailure);
            } else {
                commands.entity(entity).insert(BehaviorSuccess);
            }
        } else {
            debug_action.repeat -= 1;
        }
    }
}
```

Add your custom behavior node to app systems

```
app.add_system(debug_action::run)
```
