use bevy::{
    diagnostic::{Diagnostics, FrameTimeDiagnosticsPlugin},
    ecs::system::EntityCommands,
    prelude::*,
    reflect::TypeUuid,
    utils::HashMap,
    window::PresentMode,
};
use serde::{Deserialize, Serialize};
use simula_action::ActionPlugin;
use simula_behavior::{
    prelude::*,
    protocol::{
        BehaviorFileData, BehaviorFileId, BehaviorFileName, BehaviorProtocolClient,
        BehaviorProtocolServer, BehaviorServer,
    },
};
use simula_camera::orbitcam::*;
use simula_inspector::{InspectorPlugin, WorldInspectorPlugin};
use simula_script::{script, Scope, ScriptPlugin};
use simula_viz::{
    axes::{Axes, AxesBundle, AxesPlugin},
    grid::{Grid, GridBundle, GridPlugin},
    lines::LinesPlugin,
};

fn main() {
    App::new()
        .insert_resource(Msaa::Sample4)
        .insert_resource(ClearColor(Color::rgb(0.105, 0.10, 0.13)))
        .add_plugins(
            DefaultPlugins
                .set(WindowPlugin {
                    primary_window: Some(Window {
                        title: "[Simbotic] Simula - Scripting".to_string(),
                        resolution: (1920., 1080.).into(),
                        present_mode: PresentMode::AutoVsync,
                        fit_canvas_to_parent: true,
                        prevent_default_event_handling: false,
                        ..default()
                    }),
                    ..default()
                })
                .set(AssetPlugin {
                    watch_for_changes: true,
                    ..Default::default()
                }),
        )
        .add_plugin(InspectorPlugin)
        .add_plugin(WorldInspectorPlugin)
        .add_plugin(ActionPlugin)
        .add_plugin(FrameTimeDiagnosticsPlugin::default())
        .add_plugin(OrbitCameraPlugin)
        .add_plugin(LinesPlugin)
        .add_plugin(AxesPlugin)
        .add_plugin(GridPlugin)
        .add_plugin(ScriptPlugin)
        .add_plugin(BehaviorPlugin)
        .add_plugin(BehaviorInspectorPlugin::<DebugBehavior>::default())
        .add_system(behavior_loader::<DebugBehavior>)
        .add_system(subtree::run::<DebugBehavior>) // Subtrees are typed, need to register them separately
        .register_type::<Subtree<DebugBehavior>>()
        .insert_resource(BehaviorFiles::default())
        .add_startup_system(setup)
        .add_system(update)
        .add_system(debug_info)
        .run();
}

#[derive(Serialize, Deserialize, TypeUuid, Debug, Clone)]
#[uuid = "7CFA1742-7725-416C-B167-95DA01750E1C"]
pub enum DebugBehavior {
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
    Guard(Guard),
    Timeout(Timeout),
    Subtree(Subtree<DebugBehavior>), // Substrees are typed, this loads same tree type
}

impl Default for DebugBehavior {
    fn default() -> Self {
        Self::Debug(Debug::default())
    }
}

impl BehaviorFactory for DebugBehavior {
    fn insert(&self, commands: &mut EntityCommands) {
        match self {
            DebugBehavior::Debug(data) => BehaviorInfo::insert_with(commands, data),
            DebugBehavior::Selector(data) => BehaviorInfo::insert_with(commands, data),
            DebugBehavior::Sequencer(data) => BehaviorInfo::insert_with(commands, data),
            DebugBehavior::All(data) => BehaviorInfo::insert_with(commands, data),
            DebugBehavior::Any(data) => BehaviorInfo::insert_with(commands, data),
            DebugBehavior::Repeater(data) => BehaviorInfo::insert_with(commands, data),
            DebugBehavior::Inverter(data) => BehaviorInfo::insert_with(commands, data),
            DebugBehavior::Succeeder(data) => BehaviorInfo::insert_with(commands, data),
            DebugBehavior::Wait(data) => BehaviorInfo::insert_with(commands, data),
            DebugBehavior::Delay(data) => BehaviorInfo::insert_with(commands, data),
            DebugBehavior::Guard(data) => BehaviorInfo::insert_with(commands, data),
            DebugBehavior::Timeout(data) => BehaviorInfo::insert_with(commands, data),
            DebugBehavior::Subtree(data) => BehaviorInfo::insert_with(commands, data),
        }
    }

    fn label(&self) -> &str {
        match self {
            DebugBehavior::Debug(_) => <Debug as BehaviorInfo>::NAME,
            DebugBehavior::Selector(_) => <Selector as BehaviorInfo>::NAME,
            DebugBehavior::Sequencer(_) => <Sequencer as BehaviorInfo>::NAME,
            DebugBehavior::All(_) => <All as BehaviorInfo>::NAME,
            DebugBehavior::Any(_) => <Any as BehaviorInfo>::NAME,
            DebugBehavior::Repeater(_) => <Repeater as BehaviorInfo>::NAME,
            DebugBehavior::Inverter(_) => <Inverter as BehaviorInfo>::NAME,
            DebugBehavior::Succeeder(_) => <Succeeder as BehaviorInfo>::NAME,
            DebugBehavior::Wait(_) => <Wait as BehaviorInfo>::NAME,
            DebugBehavior::Delay(_) => <Delay as BehaviorInfo>::NAME,
            DebugBehavior::Guard(_) => <Guard as BehaviorInfo>::NAME,
            DebugBehavior::Timeout(_) => <Timeout as BehaviorInfo>::NAME,
            DebugBehavior::Subtree(_) => <Subtree<DebugBehavior> as BehaviorInfo>::NAME,
        }
    }

    fn reflect(&mut self) -> &mut dyn Reflect {
        match self {
            DebugBehavior::Debug(data) => data,
            DebugBehavior::Selector(data) => data,
            DebugBehavior::Sequencer(data) => data,
            DebugBehavior::All(data) => data,
            DebugBehavior::Any(data) => data,
            DebugBehavior::Repeater(data) => data,
            DebugBehavior::Inverter(data) => data,
            DebugBehavior::Succeeder(data) => data,
            DebugBehavior::Wait(data) => data,
            DebugBehavior::Delay(data) => data,
            DebugBehavior::Guard(data) => data,
            DebugBehavior::Timeout(data) => data,
            DebugBehavior::Subtree(data) => data,
        }
    }

    fn typ(&self) -> BehaviorType {
        match self {
            DebugBehavior::Debug(_) => <Debug as BehaviorInfo>::TYPE,
            DebugBehavior::Selector(_) => <Selector as BehaviorInfo>::TYPE,
            DebugBehavior::Sequencer(_) => <Sequencer as BehaviorInfo>::TYPE,
            DebugBehavior::All(_) => <All as BehaviorInfo>::TYPE,
            DebugBehavior::Any(_) => <Any as BehaviorInfo>::TYPE,
            DebugBehavior::Repeater(_) => <Repeater as BehaviorInfo>::TYPE,
            DebugBehavior::Inverter(_) => <Inverter as BehaviorInfo>::TYPE,
            DebugBehavior::Succeeder(_) => <Succeeder as BehaviorInfo>::TYPE,
            DebugBehavior::Wait(_) => <Wait as BehaviorInfo>::TYPE,
            DebugBehavior::Delay(_) => <Delay as BehaviorInfo>::TYPE,
            DebugBehavior::Guard(_) => <Guard as BehaviorInfo>::TYPE,
            DebugBehavior::Timeout(_) => <Timeout as BehaviorInfo>::TYPE,
            DebugBehavior::Subtree(_) => <Subtree<DebugBehavior> as BehaviorInfo>::TYPE,
        }
    }

    fn color(&self) -> Color {
        match self {
            DebugBehavior::Debug(_) => Color::hex("#235").unwrap(),
            DebugBehavior::Selector(_) => Color::hex("#522").unwrap(),
            DebugBehavior::Sequencer(_) => Color::hex("#252").unwrap(),
            DebugBehavior::All(_) => Color::hex("#252").unwrap(),
            DebugBehavior::Any(_) => Color::hex("#522").unwrap(),
            DebugBehavior::Repeater(_) => Color::hex("#440").unwrap(),
            DebugBehavior::Inverter(_) => Color::hex("#440").unwrap(),
            DebugBehavior::Succeeder(_) => Color::hex("#440").unwrap(),
            DebugBehavior::Wait(_) => Color::hex("#235").unwrap(),
            DebugBehavior::Delay(_) => Color::hex("#440").unwrap(),
            DebugBehavior::Guard(_) => Color::hex("#440").unwrap(),
            DebugBehavior::Timeout(_) => Color::hex("#440").unwrap(),
            DebugBehavior::Subtree(_) => Color::hex("#440").unwrap(),
        }
    }

    fn categories(&self) -> Vec<&'static str> {
        match self {
            DebugBehavior::Debug(_) => vec![<Debug as BehaviorInfo>::TYPE.as_ref()],
            DebugBehavior::Selector(_) => vec![<Selector as BehaviorInfo>::TYPE.as_ref()],
            DebugBehavior::Sequencer(_) => vec![<Sequencer as BehaviorInfo>::TYPE.as_ref()],
            DebugBehavior::All(_) => vec![<All as BehaviorInfo>::TYPE.as_ref()],
            DebugBehavior::Any(_) => vec![<Any as BehaviorInfo>::TYPE.as_ref()],
            DebugBehavior::Repeater(_) => vec![<Repeater as BehaviorInfo>::TYPE.as_ref()],
            DebugBehavior::Inverter(_) => vec![<Inverter as BehaviorInfo>::TYPE.as_ref()],
            DebugBehavior::Succeeder(_) => vec![<Succeeder as BehaviorInfo>::TYPE.as_ref()],
            DebugBehavior::Wait(_) => vec![<Wait as BehaviorInfo>::TYPE.as_ref()],
            DebugBehavior::Delay(_) => vec![<Delay as BehaviorInfo>::TYPE.as_ref()],
            DebugBehavior::Guard(_) => vec![<Guard as BehaviorInfo>::TYPE.as_ref()],
            DebugBehavior::Timeout(_) => vec![<Timeout as BehaviorInfo>::TYPE.as_ref()],
            DebugBehavior::Subtree(_) => {
                vec![<Subtree<DebugBehavior> as BehaviorInfo>::TYPE.as_ref()]
            }
        }
    }

    fn list() -> Vec<Self> {
        vec![
            DebugBehavior::Debug(Debug::default()),
            DebugBehavior::Selector(Selector::default()),
            DebugBehavior::Sequencer(Sequencer::default()),
            DebugBehavior::All(All::default()),
            DebugBehavior::Any(Any::default()),
            DebugBehavior::Repeater(Repeater::default()),
            DebugBehavior::Inverter(Inverter::default()),
            DebugBehavior::Succeeder(Succeeder::default()),
            DebugBehavior::Wait(Wait::default()),
            DebugBehavior::Delay(Delay::default()),
            DebugBehavior::Guard(Guard::default()),
            DebugBehavior::Timeout(Timeout::default()),
            DebugBehavior::Subtree(Subtree::<DebugBehavior>::default()),
        ]
    }
}

#[derive(Default, Resource)]
struct BehaviorFiles {
    pub files: HashMap<BehaviorFileId, BehaviorFileName>,
}

fn setup(
    mut commands: Commands,
    asset_server: Res<AssetServer>,
    mut behavior_files: ResMut<BehaviorFiles>,
    behavior_server: Res<BehaviorServer<DebugBehavior>>,
    mut scopes: ResMut<Assets<Scope>>,
    type_registry: Res<AppTypeRegistry>,
) {
    let dir_path = "assets/inspector";

    // Read the directory and handle any errors
    let paths = match std::fs::read_dir(dir_path) {
        Ok(paths) => paths,
        Err(err) => {
            eprintln!("Error reading directory: {}", err);
            return;
        }
    };

    // Iterate over the directory entries
    for path in paths {
        if let Ok(entry) = path {
            // Check if the entry is a file with the desired extension
            if entry.file_type().unwrap().is_file() {
                if let Some(file_name) = entry.file_name().to_str() {
                    if file_name.ends_with(".bht.ron") {
                        let file_name = file_name.trim_end_matches(".bht.ron");
                        behavior_files.files.insert(
                            BehaviorFileId::new(),
                            BehaviorFileName(file_name.to_owned()),
                        );
                    }
                }
            }
        }
    }

    behavior_server
        .sender
        .send(BehaviorProtocolServer::BehaviorFileNames(
            behavior_files.files.clone().into_iter().collect(),
        ))
        .unwrap();

    // // load debug behaviors
    // let behaviors = [
    //     "debug_delay",
    //     "debug_gate_true",
    //     "debug_gate_blackboard",
    //     "debug_all",
    //     "debug_any_repeat",
    //     "debug_any_subtree",
    //     "debug_any",
    //     "debug_sequence",
    //     "debug_defaults",
    //     "debug_repeater",
    //     "debug_repeat_repeater",
    //     "debug_subtree_gate",
    // ];
    // for behavior in behaviors.iter() {
    //     // get a handle to a behavior asset from asset server
    //     let behavior_handle: Handle<BehaviorAsset> =
    //         asset_server.load(format!("behaviors/{}.bht.ron", behavior).as_str());

    //     // create a new scope for the behavior tree
    //     let mut scope = Scope::new();
    //     let mut blackboard = script::Map::new();
    //     blackboard.insert("state".into(), 0.into());
    //     scope.scope.push("blackboard", blackboard);
    //     let scope_handle = scopes.add(scope);

    //     // create a new entity for the behavior tree, and insert the scope
    //     let tree_entity = commands
    //         .spawn((Name::new(format!("BT: {}", behavior)), scope_handle))
    //         .insert(simula_behavior::inspector::graph::MyGraphState {
    //             type_registry: type_registry.0.clone(),
    //             ..Default::default()
    //         })
    //         .insert(simula_behavior::inspector::graph::MyEditorState::<
    //             DebugBehavior,
    //         >::default())
    //         .id();

    //     // create a behavior tree component from the asset
    //     let behavior_tree = BehaviorTree::from_asset::<DebugBehavior>(
    //         tree_entity,
    //         None,
    //         &mut commands,
    //         behavior_handle,
    //     );

    //     // insert the behavior tree component into the tree entity and move root to tree entity
    //     if let Some(root) = behavior_tree.root {
    //         commands
    //             .entity(tree_entity)
    //             .insert(behavior_tree)
    //             .add_child(root);
    //     }
    // }

    // grid
    let grid_color = Color::rgb(0.08, 0.06, 0.08);
    commands
        .spawn(GridBundle {
            grid: Grid {
                size: 10,
                divisions: 10,
                start_color: grid_color,
                end_color: grid_color,
            },
            transform: Transform::from_translation(Vec3::new(0.0, 0.0, 0.0)),
            ..Default::default()
        })
        .insert(Name::new("Grid"));

    // axes
    commands
        .spawn(AxesBundle {
            axes: Axes {
                size: 1.,
                inner_offset: 5.,
            },
            transform: Transform::from_xyz(0.0, 0.01, 0.0),
            ..Default::default()
        })
        .insert(Name::new("Axes: World"));

    let theta = std::f32::consts::FRAC_PI_4;
    let light_transform = Mat4::from_euler(EulerRot::ZYX, 0.0, std::f32::consts::FRAC_PI_2, -theta);
    commands.spawn(DirectionalLightBundle {
        directional_light: DirectionalLight {
            color: Color::rgb(1.0, 1.0, 1.0),
            illuminance: 5000.,
            ..Default::default()
        },
        transform: Transform::from_matrix(light_transform),
        ..Default::default()
    });

    // orbit camera
    commands
        .spawn(Camera3dBundle {
            ..Default::default()
        })
        .insert(OrbitCamera {
            center: Vec3::new(0.0, 1.0, 0.0),
            distance: 10.0,
            ..Default::default()
        });

    // FPS on screen
    commands.spawn(TextBundle {
        text: Text {
            sections: vec![TextSection {
                value: "\nFPS: ".to_string(),
                style: TextStyle {
                    font: asset_server.load("fonts/FiraMono-Medium.ttf"),
                    font_size: 12.0,
                    color: Color::rgb(0.0, 1.0, 0.0),
                },
            }],
            ..Default::default()
        },
        style: Style {
            position_type: PositionType::Absolute,
            position: UiRect {
                top: Val::Px(5.0),
                left: Val::Px(5.0),
                ..Default::default()
            },
            ..Default::default()
        },
        ..Default::default()
    });
}

fn update(
    mut commands: Commands,
    asset_server: Res<AssetServer>,
    mut behavior_files: ResMut<BehaviorFiles>,
    behavior_server: Res<protocol::BehaviorServer<DebugBehavior>>,
    mut scopes: ResMut<Assets<Scope>>,
    type_registry: Res<AppTypeRegistry>,
) {
    if let Ok(client_msg) = behavior_server.receiver.try_recv() {
        match client_msg {
            BehaviorProtocolClient::LoadFile(file_id) => {
                let file_name = behavior_files.files[&file_id].clone();
                let dir_path = "assets/inspector";
                let file_ext = "bht.ron";
                let file_path = format!("{}/{}.{}", dir_path, *file_name, file_ext);
                let data = std::fs::read_to_string(file_path).unwrap();
                behavior_server
                    .sender
                    .send(BehaviorProtocolServer::BehaviorFile((
                        file_id,
                        BehaviorFileData(data.clone()),
                    )))
                    .unwrap();
            }
            _ => {
                panic!("unexpected message")
            }
        }
    }
}

fn debug_info(diagnostics: Res<Diagnostics>, mut query: Query<&mut Text>) {
    if let Some(fps) = diagnostics.get(FrameTimeDiagnosticsPlugin::FPS) {
        if let Some(average) = fps.average() {
            for mut text in query.iter_mut() {
                text.sections[0].value = format!("{:.2}", average);
            }
        }
    };
}
