use bevy::{
    diagnostic::{Diagnostics, FrameTimeDiagnosticsPlugin},
    prelude::*,
    reflect::TypeUuid,
    render::render_resource::{AsBindGroup, ShaderRef},
};
// use bevy_inspector_egui::quick::WorldInspectorPlugin;
use simula_action::ActionPlugin;
use simula_camera::flycam::*;
use simula_viz::{
    axes::{Axes, AxesBundle, AxesPlugin},
    grid::{Grid, GridBundle, GridPlugin},
    lines::LinesPlugin,
};

fn main() {
    App::new()
        .insert_resource(Msaa::Sample4)
        .insert_resource(ClearColor(Color::rgb(0.105, 0.10, 0.11)))
        .insert_resource(AmbientLight {
            color: Color::WHITE,
            brightness: 0.1,
        })
        .add_plugins(DefaultPlugins.set(WindowPlugin {
            primary_window: Some(Window {
                title: "[Simbotic] Simula - Blender".to_string(),
                resolution: (940., 528.).into(),
                present_mode: PresentMode::AutoVsync,
                fit_canvas_to_parent: true,
                prevent_default_event_handling: false,
                ..default()
            }),
            ..default()
        }))
        .add_plugin(MaterialPlugin::<FastMaterial>::default())
        .add_plugin(WorldInspectorPlugin)
        .add_plugin(ActionPlugin)
        .add_plugin(FrameTimeDiagnosticsPlugin::default())
        .add_plugin(FlyCameraPlugin)
        .add_plugin(LinesPlugin)
        .add_plugin(AxesPlugin)
        .add_plugin(GridPlugin)
        .add_startup_system(setup)
        .add_system(setup_scene_once_loaded)
        .add_system(replace_materials)
        .add_system(debug_info)
        .run();
}

#[derive(Resource)]
struct Animations(Vec<Handle<AnimationClip>>);

fn setup(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut fast_materials: ResMut<Assets<FastMaterial>>,
    asset_server: Res<AssetServer>,
) {
    // load animations
    commands.insert_resource(Animations(vec![
        asset_server.load("models/fox/Fox.glb#Animation2"),
        asset_server.load("models/fox/Fox.glb#Animation1"),
        asset_server.load("models/fox/Fox.glb#Animation0"),
    ]));

    // load gltfs

    // commands.spawn(SceneBundle {
    //     scene: asset_server.load("models/FlightHelmet/FlightHelmet.gltf#Scene0"),
    //     transform: Transform::from_xyz(0.0, 0.0, 0.0),
    //     // visibility: Visibility {
    //     //     is_visible: false,
    //     // },
    //     ..default()
    // });

    commands.spawn(SceneBundle {
        scene: asset_server.load("models/fox/Fox.glb#Scene0"),
        transform: Transform::from_scale(Vec3::splat(0.005))
            .with_translation(Vec3::new(-1.0, 0.0, 0.0)),
        ..default()
    });

    // cube
    commands
        .spawn(MaterialMeshBundle {
            mesh: meshes.add(Mesh::from(shape::Cube { size: 1.0 })),
            // material: stan_materials.add(Color::rgb(0.8, 0.7, 0.6).into()),
            material: fast_materials.add(FastMaterial {
                color: Color::rgb(0.8, 0.7, 0.6),
                ..Default::default()
            }),
            transform: Transform::from_xyz(-2.5, 0.0, -1.5),
            ..default()
        })
        .insert(Name::new("Shape: Cube"));

    // grid
    let grid_color = Color::rgb(0.08, 0.06, 0.08);
    commands
        .spawn(GridBundle {
            grid: Grid {
                size: 10,
                divisions: 10,
                start_color: grid_color,
                end_color: grid_color,
                ..Default::default()
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

    // camera
    commands
        .spawn(Camera3dBundle {
            transform: Transform::from_xyz(1.0, 0.4, 1.0).with_rotation(Quat::from_euler(
                EulerRot::YXZ,
                std::f32::consts::FRAC_PI_4,
                0.0,
                0.0,
            )),
            ..Default::default()
        })
        .insert(FlyCamera {
            accel: 2.0,
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

/// The Material trait is very configurable, but comes with sensible defaults for all methods.
/// You only need to implement functions for features that need non-default behavior. See the Material api docs for details!
impl Material for FastMaterial {
    fn fragment_shader() -> ShaderRef {
        "shaders/fast_material.wgsl".into()
    }

    fn alpha_mode(&self) -> AlphaMode {
        self.alpha_mode
    }
}

// This is the struct that will be passed to your shader
#[derive(AsBindGroup, TypeUuid, Debug, Clone, Default)]
#[uuid = "ead2eaac-a417-11ed-a6a8-02a179e5df2a"]
pub struct FastMaterial {
    #[uniform(0)]
    color: Color,
    #[texture(1)]
    #[sampler(2)]
    base_color_texture: Option<Handle<Image>>,
    alpha_mode: AlphaMode,
}

// Replace StandardMaterial with FastMaterial
fn replace_materials(
    mut commands: Commands,
    mut fast_materials: ResMut<Assets<FastMaterial>>,
    mut stan_materials: ResMut<Assets<StandardMaterial>>,
    mut query: Query<(Entity, &Handle<Mesh>, &Handle<StandardMaterial>)>,
) {
    for (entity, _mesh_handle, stan_handle) in query.iter_mut() {
        let stan_material = stan_materials.get_mut(stan_handle).unwrap();

        let fast_material = fast_materials.add(FastMaterial {
            color: stan_material.base_color,
            base_color_texture: stan_material.base_color_texture.clone(),
            alpha_mode: stan_material.alpha_mode,
        });

        commands
            .entity(entity)
            .remove::<Handle<StandardMaterial>>()
            .insert(fast_material);
    }
}

// Once the scene is loaded, start the animation
fn setup_scene_once_loaded(
    animations: Res<Animations>,
    mut player: Query<&mut AnimationPlayer>,
    mut done: Local<bool>,
) {
    if !*done {
        if let Ok(mut player) = player.get_single_mut() {
            player.play(animations.0[0].clone_weak()).repeat();
            *done = true;
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
