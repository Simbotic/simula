use bevy::{
    diagnostic::{Diagnostics, FrameTimeDiagnosticsPlugin},
    prelude::*,
};
use bevy_inspector_egui::quick::WorldInspectorPlugin;
use simula_action::ActionPlugin;
use simula_camera::orbitcam::*;
use simula_core::spline::Spline;
use simula_net::NetPlugin;
use simula_viz::{
    axes::{Axes, AxesBundle, AxesPlugin},
    grid::{Grid, GridBundle, GridPlugin},
    lines::{LineMesh, LinesMaterial, LinesPlugin},
    spline::{SplineBundle, SplinePlugin},
};

fn main() {
    App::new()
        .insert_resource(Msaa { samples: 4 })
        .insert_resource(ClearColor(Color::rgb(0.105, 0.10, 0.11)))
        .add_plugins(DefaultPlugins.set(WindowPlugin {
            window: WindowDescriptor {
                title: "[Simbotic] Simula - Splines".to_string(),
                width: 940.,
                height: 528.,
                ..default()
            },
            ..default()
        }))
        .add_plugin(NetPlugin)
        .add_plugin(WorldInspectorPlugin)
        .add_plugin(ActionPlugin)
        .add_plugin(FrameTimeDiagnosticsPlugin::default())
        .add_plugin(OrbitCameraPlugin)
        .add_plugin(LinesPlugin)
        .add_plugin(AxesPlugin)
        .add_plugin(GridPlugin)
        .add_plugin(SplinePlugin)
        .add_startup_system(setup)
        .add_system(travel_on_spline)
        .add_system(debug_info)
        .run();
}

fn setup(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut lines_materials: ResMut<Assets<LinesMaterial>>,
    line_mesh: Res<LineMesh>,
    asset_server: Res<AssetServer>,
) {
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
            mesh: meshes.add(line_mesh.clone()),
            material: lines_materials.add(LinesMaterial {}),
            transform: Transform::from_translation(Vec3::new(0.0, 0.0, 0.0)),
            ..Default::default()
        })
        .insert(Name::new("Grid"));

    // axes
    commands
        .spawn(AxesBundle {
            axes: Axes {
                size: 1.,
                inner_offset: 10.,
            },
            mesh: meshes.add(line_mesh.clone()),
            material: lines_materials.add(LinesMaterial {}),
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

    let spline = Spline::from_points(vec![
        Vec3::new(-6.0, 0.0, 0.0),
        Vec3::new(-5.0, 1.0, 0.0),
        Vec3::new(-4.0, -1.0, 0.0),
        Vec3::new(-3.0, 0.0, 0.0),
        Vec3::new(-2.0, 1.0, 0.0),
        Vec3::new(-1.0, -1.0, 0.0),
        Vec3::new(0.0, 0.0, 0.0),
        Vec3::new(1.0, 1.0, 0.0),
        Vec3::new(2.0, -1.0, 0.0),
        Vec3::new(3.0, 0.0, 0.0),
        Vec3::new(4.0, 1.0, 0.0),
        Vec3::new(5.0, -1.0, 0.0),
        Vec3::new(6.0, 0.0, 0.0),
    ]);

    // spline
    let spline = commands
        .spawn(SplineBundle {
            spline,
            mesh: meshes.add(line_mesh.clone()),
            material: lines_materials.add(LinesMaterial {}),
            transform: Transform::from_xyz(0.0, 2.0, 0.0),
            ..Default::default()
        })
        .insert(Name::new("Spline Simple"))
        .id();

    // spline traveler (time)
    commands
        .spawn(AxesBundle {
            axes: Axes {
                size: 1.0,
                inner_offset: 0.0,
            },
            mesh: meshes.add(line_mesh.clone()),
            material: lines_materials.add(LinesMaterial {}),
            transform: Transform::from_xyz(0.0, 0.0, 0.0),
            ..Default::default()
        })
        .insert(SplineTraveler {
            spline,
            travel_type: TravelType::Time,
        });

    // spline traveler (distance)
    commands
        .spawn(AxesBundle {
            axes: Axes {
                size: 1.0,
                inner_offset: 0.0,
            },
            mesh: meshes.add(line_mesh.clone()),
            material: lines_materials.add(LinesMaterial {}),
            transform: Transform::from_xyz(0.0, 0.0, 0.0),
            ..Default::default()
        })
        .insert(SplineTraveler {
            spline,
            travel_type: TravelType::Distance,
        });
}

enum TravelType {
    Time,
    Distance,
}

#[derive(Component)]
struct SplineTraveler {
    spline: Entity,
    travel_type: TravelType,
}

fn travel_on_spline(
    time: Res<Time>,
    mut query: Query<(&mut Transform, &SplineTraveler)>,
    splines: Query<(&Spline, &GlobalTransform)>,
) {
    for (mut transform, traveler) in &mut query {
        if let Ok((spline, spline_transform)) = splines.get(traveler.spline) {
            match traveler.travel_type {
                TravelType::Time => {
                    let t = time.elapsed_seconds() * 0.1;
                    let t = t % 1.0;
                    let mat = spline_transform.compute_matrix() * spline.get_frame(t);
                    *transform = Transform::from_matrix(mat);
                }
                TravelType::Distance => {
                    let d = 1.0 * time.elapsed_seconds();
                    let d = d % spline.get_length();
                    let t = spline.get_t_at_length(d);
                    let t = t % 1.0;
                    let mat = spline_transform.compute_matrix() * spline.get_frame(t);
                    *transform = Transform::from_matrix(mat);
                }
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
