use bevy::{
    diagnostic::{Diagnostics, FrameTimeDiagnosticsPlugin},
    prelude::*,
};
use rand::distributions::{Distribution, Uniform};
use simula_camera::flycam::*;
use simula_viz::{
    axes,
    grid,
    lines::{Lines, LinesPlugin},
    voxels::{Voxel, Voxels, VoxelsBundle, VoxelsPlugin},
    // voxel::{self, VoxelMaterial},
};

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
        .insert_resource(ClearColor(Color::rgb(0.125, 0.12, 0.13)))
        .add_plugins(DefaultPlugins)
        .add_plugin(FrameTimeDiagnosticsPlugin::default())
        .insert_resource(BevyCounter { count: 0 })
        .add_plugin(FlyCameraPlugin)
        .add_plugin(LinesPlugin)
        .add_plugin(VoxelsPlugin)
        .add_startup_system(setup)
        .add_system(axes::system)
        .add_system(grid::system)
        .add_system(counter_system)
        .add_system(line_test)
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
        transform: Transform::from_xyz(2.0, 0.0, 2.0),
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
    commands.spawn_bundle(grid::GridBundle {
        grid: grid::Grid {
            size: 10,
            divisions: 10,
            ..Default::default()
        },
        transform: Transform::from_translation(Vec3::new(0.0, 0.0, 0.0)),
        ..Default::default()
    });

    // axes
    commands.spawn_bundle(axes::AxesBundle {
        axes: axes::Axes {
            size: 1.,
            inner_offset: 5.,
        },
        transform: Transform::from_xyz(0.0, 0.01, 0.0),
        ..Default::default()
    });

    // x - axis
    commands
        .spawn_bundle(axes::AxesBundle {
            axes: axes::Axes {
                size: 3.,
                inner_offset: 0.,
            },
            transform: Transform::from_xyz(2.0, 0.0, 0.0),
            ..Default::default()
        })
        .insert(Rotate {
            axis: Vec3::new(1.0, 0.0, 0.0),
            angle: 1.0,
        });

    // y - axis
    commands
        .spawn_bundle(axes::AxesBundle {
            axes: axes::Axes {
                size: 3.,
                inner_offset: 0.,
            },
            transform: Transform::from_xyz(0.0, 2.0, 0.0),
            ..Default::default()
        })
        .insert(Rotate {
            axis: Vec3::new(0.0, 1.0, 0.0),
            angle: 1.0,
        });

    // z - axis
    commands
        .spawn_bundle(axes::AxesBundle {
            axes: axes::Axes {
                size: 3.,
                inner_offset: 0.,
            },
            transform: Transform::from_xyz(0.0, 0.0, 2.0),
            ..Default::default()
        })
        .insert(Rotate {
            axis: Vec3::new(0.0, 0.0, 1.0),
            angle: 1.0,
        });

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

    // voxels

    let voxels: Vec<Voxel> = vec![
        Voxel {
            position: Vec3::new(6., 0., 0.),
            size: 1.,
            color: Color::RED,
        },
        Voxel {
            position: Vec3::new(0., 6., 0.),
            size: 1.,
            color: Color::GREEN,
        },
        Voxel {
            position: Vec3::new(0., 0., 6.),
            size: 1.,
            color: Color::BLUE,
        },
    ];

    commands.spawn_bundle(VoxelsBundle {
        voxels: Voxels { voxels },
        ..Default::default()
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

fn line_test(mut lines: Query<&mut Lines>) {
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
