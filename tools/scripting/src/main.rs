use bevy::{
    diagnostic::{Diagnostics, FrameTimeDiagnosticsPlugin},
    prelude::*,
    window::PresentMode,
};
use debug_behavior::{DebugBehavior, DebugBehaviorPlugin};
use simula_action::ActionPlugin;
use simula_behavior::prelude::*;
use simula_camera::orbitcam::*;
use simula_inspector::{InspectorPlugin, WorldInspectorPlugin};
use simula_script::{script, Scope, ScriptPlugin};
use simula_viz::{
    axes::{Axes, AxesBundle, AxesPlugin},
    grid::{Grid, GridBundle, GridPlugin},
    lines::LinesPlugin,
};

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
        .add_startup_system(setup)
        .add_system(debug_info)
        // Behavior setup
        .add_plugin(BehaviorPlugin)
        .add_plugin(DebugBehaviorPlugin)
        .add_plugin(BehaviorServerPlugin::<DebugBehavior>::default())
        .add_plugin(BehaviorInspectorPlugin::<DebugBehavior>::default())
        .run();
}

fn setup(
    mut commands: Commands,
    asset_server: Res<AssetServer>,
    mut scopes: ResMut<Assets<Scope>>,
    behavior_server: Res<protocol::BehaviorServer<DebugBehavior>>,
    mut behavior_trackers: ResMut<BehaviorTrackers<DebugBehavior>>,
) {
    // load debug behaviors
    let behaviors = [
        "*bhts/debug/delay",
        "*bhts/debug/gate_true",
        "bhts/debug/gate_blackboard",
        "bhts/debug/all",
        "bhts/debug/any_repeat",
        "bhts/debug/any_subtree",
        "bhts/debug/any",
        "bhts/debug/sequence",
        "bhts/debug/defaults",
        "bhts/debug/repeater",
        "bhts/debug/repeat_repeater",
        "bhts/debug/subtree_gate",
    ];
    for behavior in behaviors.into_iter() {
        // load the behavior if it starts with *
        if let Some(behavior) = behavior.strip_prefix("*") {
            // get a handle to a behavior asset from asset server
            let behavior_handle: Handle<BehaviorAsset<DebugBehavior>> =
                asset_server.load(format!("{}.bht.ron", behavior).as_str());

            // create a new scope for the behavior tree
            let mut scope = Scope::new();
            let mut blackboard = script::Map::new();
            blackboard.insert("state".into(), 0.into());
            scope.scope.push("blackboard", blackboard);
            let scope_handle = scopes.add(scope);

            // create a new entity for the behavior tree, and insert the scope
            let tree_entity = commands
                .spawn((Name::new(format!("BHT: {}", behavior)), scope_handle))
                .id();
            BehaviorTree::build_tree_from_asset(tree_entity, &mut commands, behavior_handle);
        }
        // do not load just yet, track file and send file name to client
        else {
            let file_id = protocol::BehaviorFileId::new();
            let file_name = protocol::BehaviorFileName(behavior.into());
            behavior_trackers.insert(
                file_id.clone(),
                BehaviorTracker {
                    file_name: file_name.clone(),
                    entity: None,
                    telemetry: false,
                    asset: None,
                },
            );
            behavior_server
                .sender
                .send(protocol::BehaviorProtocolServer::FileName(
                    file_id, file_name,
                ))
                .unwrap();
        }
    }

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
                bottom: Val::Px(5.0),
                right: Val::Px(5.0),
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
                text.sections[0].value = format!("{:.0}", average);
            }
        }
    };
}
