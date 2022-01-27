use std::f32::consts::PI;

use bevy::{
    diagnostic::{Diagnostics, FrameTimeDiagnosticsPlugin},
    prelude::*,
    // render::wireframe::{Wireframe, WireframeConfig, WireframePlugin},
    // wgpu::{WgpuFeature, WgpuFeatures, WgpuOptions},
};
use rand::{
    distributions::{Distribution, Uniform},
    Rng,
};
use simula_camera::flycam::*;
use simula_viz::lines::{Lines, LinesBundle, LinesMaterial, LinesPlugin};

// mod sandbox;

fn main() {
    App::new()
        .insert_resource(WindowDescriptor {
            title: "[Simbotic] Simula - Sandbox".to_string(),
            width: 800.,
            height: 600.,
            vsync: false,
            ..Default::default()
        })
        .insert_resource(Msaa { samples: 4 })
        .insert_resource(ClearColor(Color::rgb(0.15, 0.15, 0.17)))
        .add_plugins(DefaultPlugins)
        // .add_plugin(WireframePlugin)
        .add_plugin(FrameTimeDiagnosticsPlugin::default())
        .insert_resource(BevyCounter { count: 0 })
        .add_plugin(FlyCameraPlugin)
        .add_plugin(LinesPlugin)
        // .insert_resource(Lines::default())
        // .insert_resource(line::Lines {
        //     depth_test: true,
        //     ..Default::default()
        // })
        .add_startup_system(setup)
        .add_system(counter_system)
        .add_system(line_test)
        // .add_system(axes::system)
        // .insert_resource(sandbox::World::new())
        // .add_startup_system(sandbox::setup)
        .run();
}

fn setup(
    mut commands: Commands,
    // mut wireframe_config: ResMut<WireframeConfig>,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    asset_server: Res<AssetServer>,
) {
    // wireframe_config.global = false;

    // tch::maybe_init_cuda();

    commands.spawn_bundle(LinesBundle::default());
    // commands.spawn_bundle(LinesBundle::default());

    // plane
    commands.spawn_bundle(PbrBundle {
        mesh: meshes.add(Mesh::from(shape::Plane { size: 5.0 })),
        material: materials.add(Color::rgb(0.3, 0.5, 0.3).into()),
        ..Default::default()
    });
    // cube
    // commands.spawn_bundle(PbrBundle {
    //     mesh: meshes.add(Mesh::from(shape::Cube { size: 1.0 })),
    //     material: materials.add(Color::rgb(0.8, 0.7, 0.6).into()),
    //     transform: Transform::from_xyz(0.0, 0.5, 0.0),
    //     ..Default::default()
    // });

    // lights
    // commands.spawn_bundle(PointLightBundle {
    //     transform: Transform::from_xyz(400.0, 5000.0, 400.0),
    //     point_light: PointLight {
    //         intensity: 50000.,
    //         color: Color::rgb(0.0, 0.0, 1.0),
    //         range: 100000.,
    //         ..Default::default()
    //     },
    //     ..Default::default()
    // });

    // Transform::from_xyz(0.0, -100.0, -1.0),
    // GlobalTransform::identity(),

    // commands.spawn().insert_bundle((
    //     meshes.add(Mesh::from(shape::Cube { size: 1.0 })),
    //     Transform::from_xyz(0.0, 0.5, 0.0),
    //     GlobalTransform::default(),
    //     LinesMaterial,
    //     Visibility::default(),
    //     ComputedVisibility::default(),
    // ));

    let theta = std::f32::consts::FRAC_PI_4;
    let light_transform = Mat4::from_euler(EulerRot::ZYX, 0.0, std::f32::consts::FRAC_PI_2, -theta);
    commands.spawn_bundle(DirectionalLightBundle {
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
        .spawn_bundle(PerspectiveCameraBundle {
            transform: Transform::from_xyz(0.0, 1.5, 8.0),
            ..Default::default()
        })
        .insert(FlyCamera {
            sensitivity: 100.,
            ..Default::default()
        });

    commands.spawn_bundle(UiCameraBundle::default());

    commands.spawn_bundle(TextBundle {
        text: Text {
            sections: vec![
                TextSection {
                    value: "Agent Count: ".to_string(),
                    style: TextStyle {
                        font: asset_server.load("fonts/FiraMono-Medium.ttf"),
                        font_size: 12.0,
                        color: Color::rgb(0.0, 1.0, 0.0),
                    },
                },
                TextSection {
                    value: "".to_string(),
                    style: TextStyle {
                        font: asset_server.load("fonts/FiraMono-Medium.ttf"),
                        font_size: 12.0,
                        color: Color::rgb(0.0, 1.0, 1.0),
                    },
                },
                TextSection {
                    value: "\nAverage FPS: ".to_string(),
                    style: TextStyle {
                        font: asset_server.load("fonts/FiraMono-Medium.ttf"),
                        font_size: 12.0,
                        color: Color::rgb(0.0, 1.0, 0.0),
                    },
                },
                TextSection {
                    value: "".to_string(),
                    style: TextStyle {
                        font: asset_server.load("fonts/FiraMono-Medium.ttf"),
                        font_size: 12.0,
                        color: Color::rgb(0.0, 1.0, 1.0),
                    },
                },
            ],
            ..Default::default()
        },
        style: Style {
            position_type: PositionType::Absolute,
            position: Rect {
                top: Val::Px(5.0),
                left: Val::Px(5.0),
                ..Default::default()
            },
            ..Default::default()
        },
        ..Default::default()
    });

    // commands
    //     .spawn_bundle((
    //         Transform::from_xyz(0.0, -100.0, -1.0),
    //         GlobalTransform::identity(),
    //     ))
    //     // .with_children(|parent| {
    //     //     parent.spawn_scene(asset_server.load("models/DesertV2/desert.gltf#Scene0"));
    //     // })
    //     .insert(Wireframe);
}

struct BevyCounter {
    pub count: u128,
}

fn counter_system(
    diagnostics: Res<Diagnostics>,
    counter: Res<BevyCounter>,
    mut query: Query<&mut Text>,
) {
    if let Some(fps) = diagnostics.get(FrameTimeDiagnosticsPlugin::FPS) {
        if let Some(average) = fps.average() {
            for mut text in query.iter_mut() {
                text.sections[1].value = format!("{}", counter.count);
                text.sections[3].value = format!("{:.2}", average);
            }
        }
    };
}

fn line_test(mut lines: Query<&mut Lines, With<LinesMaterial>>) {
    let mut rng = rand::thread_rng();
    let die = Uniform::from(0f32..1f32);

    for mut lines in lines.iter_mut() {
        for _ in 0..20 {
            let start = Vec3::new(
                -die.sample(&mut rng),
                -die.sample(&mut rng),
                -die.sample(&mut rng),
            );
            let end = Vec3::new(
                die.sample(&mut rng),
                die.sample(&mut rng),
                die.sample(&mut rng),
            );

            let color = Color::Hsla {
                hue: die.sample(&mut rng) * 360.0,
                lightness: 0.5,
                saturation: 1.0,
                alpha: 1.0,
            };
            lines.line_colored(start, end, color);
        }

        lines.line_gradient(
            Vec3::new(-10.0, 0.0, 0.0),
            Vec3::new(10.0, 0.0, 0.0),
            Color::DARK_GRAY,
            Color::RED,
        );
        lines.line_gradient(
            Vec3::new(0.0, -10.0, 0.0),
            Vec3::new(0.0, 10.0, 0.0),
            Color::DARK_GRAY,
            Color::GREEN,
        );
        lines.line_gradient(
            Vec3::new(0.0, 0.0, -10.0),
            Vec3::new(0.0, 0.0, 10.0),
            Color::DARK_GRAY,
            Color::BLUE,
        );
    }
}
