use bevy::{
    diagnostic::{Diagnostics, FrameTimeDiagnosticsPlugin},
    prelude::*,
};
use bevy_inspector_egui::WorldInspectorPlugin;
use rand::distributions::{Distribution, Uniform};
use simula_camera::flycam::*;
use simula_core::signal;
use simula_viz::{
    axes::{Axes, AxesBundle, AxesPlugin},
    grid::{Grid, GridBundle, GridPlugin},
    lines::{Lines, LinesBundle, LinesPlugin},
    voxels::{Voxel, Voxels, VoxelsBundle, VoxelsPlugin},
};

fn main() {
    App::new()
        .register_type::<signal::Generator>()
        .register_type::<signal::Function>()
        .register_type::<signal::Controller<f32>>()
        .insert_resource(WindowDescriptor {
            title: "[Simbotic] Simula - Sandbox".to_string(),
            width: 940.,
            height: 528.,
            vsync: false,
            ..Default::default()
        })
        .insert_resource(Msaa { samples: 4 })
        .insert_resource(ClearColor(Color::rgb(0.125, 0.12, 0.13)))
        .add_plugins(DefaultPlugins)
        .add_plugin(WorldInspectorPlugin::new())
        .add_plugin(FrameTimeDiagnosticsPlugin::default())
        .add_plugin(FlyCameraPlugin)
        .add_plugin(LinesPlugin)
        .add_plugin(AxesPlugin)
        .add_plugin(GridPlugin)
        .add_plugin(VoxelsPlugin)
        .add_startup_system(setup)
        .add_system(debug_info)
        .add_system(line_test)
        .add_system(line_signal_generator)
        .add_system(line_signal_control)
        .add_system(rotate_system)
        .run();
}

fn setup(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    asset_server: Res<AssetServer>,
) {
    // plane
    commands.spawn_bundle(PbrBundle {
        mesh: meshes.add(Mesh::from(shape::Plane { size: 1.0 })),
        material: materials.add(Color::rgb(0.3, 0.5, 0.3).into()),
        transform: Transform::from_xyz(2.0, 0.01, 2.0),
        ..Default::default()
    });

    // cube
    commands.spawn_bundle(PbrBundle {
        mesh: meshes.add(Mesh::from(shape::Cube { size: 1.0 })),
        material: materials.add(Color::rgb(0.8, 0.7, 0.6).into()),
        transform: Transform::from_xyz(-2.0, 0.0, -2.0),
        ..Default::default()
    });

    // grid
    commands.spawn_bundle(GridBundle {
        grid: Grid {
            size: 10,
            divisions: 10,
            ..Default::default()
        },
        transform: Transform::from_translation(Vec3::new(0.0, 0.0, 0.0)),
        ..Default::default()
    });

    // axes
    commands.spawn_bundle(AxesBundle {
        axes: Axes {
            size: 1.,
            inner_offset: 5.,
        },
        transform: Transform::from_xyz(0.0, 0.01, 0.0),
        ..Default::default()
    });

    // x - axis
    commands
        .spawn_bundle(AxesBundle {
            axes: Axes {
                size: 3.,
                inner_offset: 0.,
            },
            transform: Transform::from_xyz(7.0, 0.0, 0.0),
            ..Default::default()
        })
        .insert(Rotate {
            axis: Vec3::new(1.0, 0.0, 0.0),
            angle: 1.0,
        })
        .insert(RandomLines);

    // y - axis
    commands
        .spawn_bundle(AxesBundle {
            axes: Axes {
                size: 3.,
                inner_offset: 0.,
            },
            transform: Transform::from_xyz(0.0, 7.0, 0.0),
            ..Default::default()
        })
        .insert(Rotate {
            axis: Vec3::new(0.0, 1.0, 0.0),
            angle: 1.0,
        })
        .insert(RandomLines);

    // z - axis
    commands
        .spawn_bundle(AxesBundle {
            axes: Axes {
                size: 3.,
                inner_offset: 0.,
            },
            transform: Transform::from_xyz(0.0, 0.0, -7.0),
            ..Default::default()
        })
        .insert(Rotate {
            axis: Vec3::new(0.0, 0.0, 1.0),
            angle: 1.0,
        })
        .insert(RandomLines);

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
            position: Rect {
                top: Val::Px(5.0),
                left: Val::Px(5.0),
                ..Default::default()
            },
            ..Default::default()
        },
        ..Default::default()
    });

    // voxels

    let voxels: Vec<Voxel> = vec![
        Voxel {
            position: Vec3::new(6., 0., 0.),
            size: 0.5,
            color: Color::RED,
        },
        Voxel {
            position: Vec3::new(0., 6., 0.),
            size: 0.5,
            color: Color::GREEN,
        },
        Voxel {
            position: Vec3::new(0., 0., -6.),
            size: 0.5,
            color: Color::rgba(0.0, 0.0, 1.0, 0.2),
        },
    ];

    commands
        .spawn_bundle(VoxelsBundle {
            voxels: Voxels { voxels },
            ..Default::default()
        })
        .insert(Rotate {
            axis: Vec3::new(0.0, 1.0, 0.0),
            angle: 1.0,
        });

    let rod_mesh = simula_viz::rod::Rod {
        ..Default::default()
    };
    let rod_mesh = simula_viz::rod::RodMesh::from(rod_mesh);

    commands.spawn().insert_bundle(PbrBundle {
        mesh: meshes.add(rod_mesh.mesh),
        material: materials.add(StandardMaterial {
            base_color: Color::PINK,
            ..Default::default()
        }),
        transform: Transform::from_xyz(5.0, 0.0, -5.0),
        ..Default::default()
    });

    commands.spawn_scene(asset_server.load("models/metric_plane/metric_plane_8x8.gltf#Scene0"));

    commands
        .spawn()
        .insert_bundle((
            Transform::from_xyz(-2.5, 0.0, 2.5),
            GlobalTransform::default(),
        ))
        .with_children(|parent| {
            parent.spawn_scene(asset_server.load("models/metric_box/metric_box_1x1.gltf#Scene0"));
        });

    // generator signals

    let points: Vec<Vec3> = (-100i32..=100)
        .map(|i| Vec3::new((i as f32) * 0.01, 0.0, 0.0))
        .collect();

    commands
        .spawn_bundle(LinesBundle {
            transform: Transform::from_xyz(0.0, 3.0, 0.0),
            ..Default::default()
        })
        .insert(signal::Generator {
            func: signal::Function::Sine,
            amplitude: 0.1,
            frequency: 3.0,
            ..Default::default()
        })
        .insert(SignalGeneratorLine {
            points: points.clone(),
        });

    commands
        .spawn_bundle(LinesBundle {
            transform: Transform::from_xyz(0.0, 2.8, 0.0),
            ..Default::default()
        })
        .insert(signal::Generator {
            func: signal::Function::Square,
            amplitude: 0.1,
            frequency: 3.0,
            ..Default::default()
        })
        .insert(SignalGeneratorLine {
            points: points.clone(),
        });

    commands
        .spawn_bundle(LinesBundle {
            transform: Transform::from_xyz(0.0, 2.6, 0.0),
            ..Default::default()
        })
        .insert(signal::Generator {
            func: signal::Function::Triangle,
            amplitude: 0.1,
            frequency: 3.0,
            ..Default::default()
        })
        .insert(SignalGeneratorLine {
            points: points.clone(),
        });

    commands
        .spawn_bundle(LinesBundle {
            transform: Transform::from_xyz(0.0, 2.4, 0.0),
            ..Default::default()
        })
        .insert(signal::Generator {
            func: signal::Function::Sawtooth,
            amplitude: 0.1,
            frequency: 3.0,
            ..Default::default()
        })
        .insert(SignalGeneratorLine {
            points: points.clone(),
        });

    commands
        .spawn_bundle(LinesBundle {
            transform: Transform::from_xyz(0.0, 2.2, 0.0),
            ..Default::default()
        })
        .insert(signal::Generator {
            func: signal::Function::Pulse,
            amplitude: 0.1,
            frequency: 3.0,
            ..Default::default()
        })
        .insert(SignalGeneratorLine {
            points: points.clone(),
        });

    commands
        .spawn_bundle(LinesBundle {
            transform: Transform::from_xyz(0.0, 2.0, 0.0),
            ..Default::default()
        })
        .insert(signal::Generator {
            func: signal::Function::WhiteNoise,
            amplitude: 0.1,
            frequency: 3.0,
            ..Default::default()
        })
        .insert(SignalGeneratorLine {
            points: points.clone(),
        });

    commands
        .spawn_bundle(LinesBundle {
            transform: Transform::from_xyz(0.0, 1.8, 0.0),
            ..Default::default()
        })
        .insert(signal::Generator {
            func: signal::Function::GaussNoise,
            amplitude: 0.1,
            frequency: 3.0,
            ..Default::default()
        })
        .insert(SignalGeneratorLine {
            points: points.clone(),
        });

    commands
        .spawn_bundle(LinesBundle {
            transform: Transform::from_xyz(0.0, 1.6, 0.0),
            ..Default::default()
        })
        .insert(signal::Generator {
            func: signal::Function::DigitalNoise,
            amplitude: 0.1,
            frequency: 3.0,
            ..Default::default()
        })
        .insert(SignalGeneratorLine {
            points: points.clone(),
        });

    // control signals

    commands
        .spawn_bundle(LinesBundle {
            transform: Transform::from_xyz(0.0, 4.0, 0.0),
            ..Default::default()
        })
        .insert(signal::Generator {
            func: signal::Function::Pulse,
            amplitude: 1.0,
            frequency: 1.0,
            ..Default::default()
        })
        .insert(SignalGeneratorLine {
            points: points.clone(),
        })
        .insert(signal::Controller::<f32> {
            kp: 0.1,
            ki: 0.0,
            kd: 0.0,
            ..Default::default()
        })
        .insert(SignalControlLine {
            points: points.clone(),
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

#[derive(Component)]
struct RandomLines;

#[derive(Component)]
struct Rotate {
    axis: Vec3,
    angle: f32,
}

fn rotate_system(time: Res<Time>, mut query: Query<(&Rotate, &mut Transform)>) {
    for (rotate, mut transform) in query.iter_mut() {
        transform.rotate(Quat::from_axis_angle(
            rotate.axis,
            rotate.angle * time.delta_seconds(),
        ));
    }
}

fn line_test(mut lines: Query<&mut Lines, With<RandomLines>>) {
    let mut rng = rand::thread_rng();
    let die = Uniform::from(0f32..1f32);

    for mut lines in lines.iter_mut() {
        for _ in 0..20 {
            let x = die.sample(&mut rng) * 0.2 - 0.1;
            let z = die.sample(&mut rng) * 0.2 - 0.1;
            let start = Vec3::new(x, -0.1, z);
            let end = Vec3::new(x, 0.1, z);

            let color = Color::Hsla {
                hue: die.sample(&mut rng) * 360.0,
                lightness: 0.5,
                saturation: 1.0,
                alpha: 1.0,
            };
            lines.line_colored(start, end, color);
        }
    }
}

#[derive(Component)]
struct SignalGeneratorLine {
    points: Vec<Vec3>,
}

fn line_signal_generator(
    time: Res<Time>,
    mut signals: Query<(&mut signal::Generator, &mut SignalGeneratorLine, &mut Lines)>,
) {
    let mut hue = 0.0;
    let hue_dt = 360.0 / signals.iter().count() as f32;
    for (mut generator, mut signal_line, mut lines) in signals.iter_mut() {
        let num_points = signal_line.points.len();
        for i in 0..(num_points - 1) {
            signal_line.points[i].y = signal_line.points[i + 1].y;
        }

        signal_line.points[num_points - 1].y = generator.sample(time.time_since_startup());

        let color = Color::Hsla {
            hue,
            lightness: 0.5,
            saturation: 1.0,
            alpha: 1.0,
        };

        for i in 0..(num_points - 1) {
            let start = signal_line.points[i];
            let end = signal_line.points[i + 1];
            lines.line_colored(start, end, color);
        }

        hue = hue + hue_dt;
    }
}

#[derive(Component)]
struct SignalControlLine {
    points: Vec<Vec3>,
}

fn line_signal_control(
    time: Res<Time>,
    mut signals: Query<(
        &mut signal::Controller<f32>,
        &SignalGeneratorLine,
        &mut SignalControlLine,
        &mut Lines,
    )>,
) {
    let mut hue = 100.0;
    let hue_dt = 360.0 / signals.iter().count() as f32;
    for (mut controller, signal_line, mut control_line, mut lines) in signals.iter_mut() {
        let num_points = control_line.points.len();
        for i in 0..(num_points - 1) {
            control_line.points[i].y = control_line.points[i + 1].y;
        }

        let control = controller.control(
            signal_line.points[num_points - 1].y,
            control_line.points[num_points - 1].y,
            time.delta(),
        );
        control_line.points[num_points - 1].y = control_line.points[num_points - 1].y + control;

        let color = Color::Hsla {
            hue,
            lightness: 0.5,
            saturation: 1.0,
            alpha: 1.0,
        };

        for i in 0..(num_points - 1) {
            let start = control_line.points[i];
            let end = control_line.points[i + 1];
            lines.line_colored(start, end, color);
        }

        hue = hue + hue_dt;
    }
}
