use behavior_server::BehaviorServerPlugin;
use bevy::{
    diagnostic::{Diagnostics, FrameTimeDiagnosticsPlugin},
    prelude::*,
    // utils::HashMap,
    window::PresentMode,
};
use debug_behavior::DebugBehavior;
use simula_action::ActionPlugin;
use simula_behavior::prelude::*;
use simula_camera::orbitcam::*;
use simula_inspector::{InspectorPlugin, WorldInspectorPlugin};
use simula_script::{Scope, ScriptPlugin};
use simula_viz::{
    axes::{Axes, AxesBundle, AxesPlugin},
    grid::{Grid, GridBundle, GridPlugin},
    lines::LinesPlugin,
};

mod behavior_server;
mod debug_behavior;

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
        .add_plugin(BehaviorServerPlugin::<DebugBehavior>::default())
        .add_startup_system(setup::<DebugBehavior>)
        .add_system(debug_info)
        .run();
}

fn setup<T: BehaviorFactory>(
    mut commands: Commands,
    asset_server: Res<AssetServer>,
    mut _scopes: ResMut<Assets<Scope>>,
    _type_registry: Res<AppTypeRegistry>,
) {
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

fn debug_info(diagnostics: Res<Diagnostics>, mut query: Query<&mut Text>) {
    if let Some(fps) = diagnostics.get(FrameTimeDiagnosticsPlugin::FPS) {
        if let Some(average) = fps.average() {
            for mut text in query.iter_mut() {
                text.sections[0].value = format!("{:.2}", average);
            }
        }
    };
}
