use bevy::{
    diagnostic::{Diagnostics, FrameTimeDiagnosticsPlugin},
    prelude::*,
    window::PresentMode,
};
use simula_action::ActionPlugin;
use simula_behavior::prelude::*;
use simula_camera::orbitcam::*;
use simula_inspector::{InspectorPlugin, WorldInspectorPlugin};
use simula_viz::{
    axes::{Axes, AxesBundle, AxesPlugin},
    grid::{Grid, GridBundle, GridPlugin},
    lines::LinesPlugin,
};

use derived_behavior::{DerivedBehavior, DerivedBehaviorPlugin};
use implemented_behavior::{ImplementedBehavior, ImplementedBehaviorPlugin};

mod derived_behavior;
mod implemented_behavior;

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
        .add_system(debug_info)
        .add_startup_system(scene_setup)
        // Behavior setup
        .add_plugin(BehaviorPlugin)
        // DebugBehavior setup
        .add_plugin(ImplementedBehaviorPlugin)
        .add_plugin(BehaviorServerPlugin::<ImplementedBehavior>::default())
        .add_plugin(BehaviorInspectorPlugin::<ImplementedBehavior>::default())
        .add_startup_system(behavior_setup::<ImplementedBehavior>)
        // DerivedBehavior setup
        .add_plugin(DerivedBehaviorPlugin)
        .add_plugin(BehaviorServerPlugin::<DerivedBehavior>::default())
        .add_plugin(BehaviorInspectorPlugin::<DerivedBehavior>::default())
        .add_startup_system(behavior_setup::<DerivedBehavior>)
        .run();
}

fn behavior_setup<T: BehaviorFactory>(
    mut commands: Commands,
    asset_server: Res<AssetServer>,
    behavior_server: Res<protocol::BehaviorServer<T>>,
    mut behavior_trackers: ResMut<BehaviorTrackers<T>>,
) {
    // load debug behaviors
    let behaviors = [
        "bht/d/delay",
        "bht/d/debug",
        "bht/d/gate_true",
        "bht/d/gate_blackboard",
        "bht/d/all",
        "bht/d/any_repeat",
        "bht/d/any_subtree",
        "bht/d/any",
        "bht/d/sequence",
        "bht/d/defaults",
        "bht/d/repeater",
        "bht/d/repeat_repeater",
        "bht/d/subtree_typed",
        "bht/d/subtree_gate",
        "bht/d/zero_timers",
        "?dynamic_01",
        "?dynamic_02",
        "?dynamic_03",
    ];
    for behavior in behaviors.into_iter() {
        // create a behavior tree that will automatically load and run
        if let Some(behavior) = behavior.strip_prefix("*") {
            // get a handle to a behavior asset from asset server
            let behavior_handle: Handle<BehaviorDocument> =
                asset_server.load(format!("{}.bht.ron", behavior).as_str());

            // create a new entity for the behavior tree
            commands.spawn((
                Name::new(format!("BHT: {}", behavior)),
                behavior_handle.clone(),
                BehaviorTree::<T>::default(),
                BehaviorTreeReset::<T>::default(),
            ));
        }
        // create behavior tree without behavior asset, it will dynamically be added later
        else if let Some(behavior) = behavior.strip_prefix("?") {
            // create a new entity for the behavior tree, and insert the scope
            commands.spawn((
                Name::new(format!("BHT: {}", behavior)),
                BehaviorTree::<T>::default(),
            ));
        }
        // do not load just yet, track file and send file name to client
        else {
            let file_id = protocol::BehaviorFileId::new();
            let file_name = protocol::BehaviorFileName(behavior.into());
            behavior_trackers.insert(
                file_id.clone(),
                BehaviorTracker {
                    file_name: file_name.clone(),
                    entity: EntityTracker::None,
                    asset: AssetTracker::None,
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
}

fn scene_setup(mut commands: Commands, asset_server: Res<AssetServer>) {
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
