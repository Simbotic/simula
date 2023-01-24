use bevy::{
    diagnostic::{Diagnostics, FrameTimeDiagnosticsPlugin},
    math::Vec3A,
    prelude::*,
    render::{
        mesh::{Indices, VertexAttributeValues},
        render_resource::PrimitiveTopology,
    },
};
use bevy_inspector_egui::quick::WorldInspectorPlugin;
use simula_action::ActionPlugin;
use simula_camera::orbitcam::*;
use simula_sdf::{
    ndshape::{ConstShape, ConstShape3u32},
    surface::{surface_nets, SurfaceNetsBuffer},
};
use simula_viz::{
    axes::{Axes, AxesBundle, AxesPlugin},
    grid::{Grid, GridBundle, GridPlugin},
    lines::{LineMesh, LinesMaterial, LinesPlugin},
};

fn main() {
    App::new()
        .insert_resource(Msaa { samples: 4 })
        .insert_resource(ClearColor(Color::rgb(0.105, 0.10, 0.11)))
        .add_plugins(DefaultPlugins.set(WindowPlugin {
            window: WindowDescriptor {
                title: "[Simbotic] Simula - SDF CSG".to_string(),
                width: 940.,
                height: 528.,
                ..default()
            },
            ..default()
        }))
        .add_plugin(WorldInspectorPlugin)
        .add_plugin(ActionPlugin)
        .add_plugin(FrameTimeDiagnosticsPlugin::default())
        .add_plugin(OrbitCameraPlugin)
        .add_plugin(LinesPlugin)
        .add_plugin(AxesPlugin)
        .add_plugin(GridPlugin)
        .add_startup_system(setup)
        .add_system(debug_info)
        .run();
}

fn setup(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut lines_materials: ResMut<Assets<LinesMaterial>>,
    line_mesh: Res<LineMesh>,
    asset_server: Res<AssetServer>,
    mut materials: ResMut<Assets<StandardMaterial>>,
) {
    let (_sphere_buffer, sphere_mesh) = sdf_to_mesh(&mut meshes, |p| sphere(0.9, p));
    let (_cube_buffer, cube_mesh) = sdf_to_mesh(&mut meshes, |p| cube(Vec3A::splat(0.5), p));
    let (_link_buffer, link_mesh) = sdf_to_mesh(&mut meshes, |p| link(0.26, 0.4, 0.18, p));

    spawn_pbr(
        &mut commands,
        &mut materials,
        sphere_mesh,
        Transform::from_translation(Vec3::new(-16.0, -16.0, -16.0)),
    );
    spawn_pbr(
        &mut commands,
        &mut materials,
        cube_mesh,
        Transform::from_translation(Vec3::new(-16.0, -16.0, 16.0)),
    );
    spawn_pbr(
        &mut commands,
        &mut materials,
        link_mesh,
        Transform::from_translation(Vec3::new(16.0, -16.0, -16.0)),
    );

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
                inner_offset: 5.,
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

fn into_domain(array_dim: u32, [x, y, z]: [u32; 3]) -> Vec3A {
    (2.0 / array_dim as f32) * Vec3A::new(x as f32, y as f32, z as f32) - 1.0
}

fn sphere(radius: f32, p: Vec3A) -> f32 {
    p.length() - radius
}

fn cube(b: Vec3A, p: Vec3A) -> f32 {
    let q = p.abs() - b;
    q.max(Vec3A::ZERO).length() + q.max_element().min(0.0)
}

fn link(le: f32, r1: f32, r2: f32, p: Vec3A) -> f32 {
    let q = Vec3A::new(p.x, (p.y.abs() - le).max(0.0), p.z);
    Vec2::new(q.length() - r1, q.z).length() - r2
}

fn sdf_to_mesh(
    meshes: &mut Assets<Mesh>,
    sdf: impl Fn(Vec3A) -> f32,
) -> (SurfaceNetsBuffer, Handle<Mesh>) {
    type SampleShape = ConstShape3u32<34, 34, 34>;

    let mut samples = [1.0; SampleShape::SIZE as usize];
    for i in 0u32..(SampleShape::SIZE) {
        let p = into_domain(32, SampleShape::delinearize(i));
        samples[i as usize] = sdf(p);
    }

    let mut buffer = SurfaceNetsBuffer::default();
    surface_nets(&samples, &SampleShape {}, [0; 3], [33; 3], &mut buffer);

    let num_vertices = buffer.positions.len();

    let mut render_mesh = Mesh::new(PrimitiveTopology::TriangleList);
    render_mesh.insert_attribute(
        Mesh::ATTRIBUTE_POSITION,
        VertexAttributeValues::Float32x3(buffer.positions.clone()),
    );
    render_mesh.insert_attribute(
        Mesh::ATTRIBUTE_NORMAL,
        VertexAttributeValues::Float32x3(buffer.normals.clone()),
    );
    render_mesh.insert_attribute(
        Mesh::ATTRIBUTE_UV_0,
        VertexAttributeValues::Float32x2(vec![[0.0; 2]; num_vertices]),
    );
    render_mesh.set_indices(Some(Indices::U32(buffer.indices.clone())));

    (buffer, meshes.add(render_mesh))
}

fn spawn_pbr(
    commands: &mut Commands,
    materials: &mut Assets<StandardMaterial>,
    mesh: Handle<Mesh>,
    transform: Transform,
) {
    let mut material = StandardMaterial::from(Color::rgb(0.0, 0.0, 0.0));
    material.perceptual_roughness = 0.9;

    commands.spawn(PbrBundle {
        mesh,
        material: materials.add(material),
        transform,
        ..Default::default()
    });
}
