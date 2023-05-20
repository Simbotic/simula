use bevy::{
    diagnostic::{Diagnostics, FrameTimeDiagnosticsPlugin},
    ecs::system::EntityCommands,
    prelude::*,
    reflect::TypeUuid,
    window::PresentMode,
};
use bevy_inspector_egui::quick::WorldInspectorPlugin;
use serde::{Deserialize, Serialize};
use simula_action::ActionPlugin;
use simula_behavior::prelude::*;
use simula_camera::orbitcam::*;
use simula_script::ScriptPlugin;
use simula_viz::{
    axes::{Axes, AxesBundle, AxesPlugin},
    grid::{Grid, GridBundle, GridPlugin},
    lines::LinesPlugin,
};

fn main() {
    App::new()
        .insert_resource(Msaa::Sample4)
        .insert_resource(ClearColor(Color::rgb(0.105, 0.10, 0.11)))
        .add_plugins(
            DefaultPlugins
                .set(WindowPlugin {
                    primary_window: Some(Window {
                        title: "[Simbotic] Simula - Scripting".to_string(),
                        resolution: (940., 528.).into(),
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
        .add_plugin(WorldInspectorPlugin::default())
        .add_plugin(ActionPlugin)
        .add_plugin(FrameTimeDiagnosticsPlugin::default())
        .add_plugin(OrbitCameraPlugin)
        .add_plugin(LinesPlugin)
        .add_plugin(AxesPlugin)
        .add_plugin(GridPlugin)
        .add_plugin(ScriptPlugin)
        .add_plugin(BehaviorPlugin)
        .add_plugin(BehaviorInspectorPlugin)
        .add_startup_system(setup)
        .add_system(debug_info)
        .add_system(behavior_loader::<DebugBehavior>)
        // Subtrees are typed, so we need to register them separately
        .add_system(subtree::run::<DebugBehavior>)
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
    Gate(Gate),
    // Substrees are typed, let's allow loading this same type as subtree
    Subtree(Subtree<DebugBehavior>),
}

impl Default for DebugBehavior {
    fn default() -> Self {
        Self::Debug(Debug::default())
    }
}

impl BehaviorSpawner for DebugBehavior {
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
            DebugBehavior::Gate(data) => BehaviorInfo::insert_with(commands, data),
            DebugBehavior::Subtree(data) => BehaviorInfo::insert_with(commands, data),
        }
    }
}

fn setup(mut commands: Commands, asset_server: Res<AssetServer>) {
    // load debug behaviors
    let behaviors = [
        "debug_delay",
        "debug_gate",
        "debug_all",
        "debug_any_repeat",
        "debug_any_subtree",
        "debug_any",
        "debug_sequence",
        "debug_defaults",
        "debug_repeater",
    ];
    for behavior in behaviors.iter() {
        let behavior_handle: Handle<BehaviorAsset> =
            asset_server.load(format!("behaviors/{}.bht.ron", behavior).as_str());
        let behavior_tree =
            BehaviorTree::from_asset::<DebugBehavior>(None, &mut commands, behavior_handle);
        commands.spawn((behavior_tree, Name::new(behavior.to_string())));
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
