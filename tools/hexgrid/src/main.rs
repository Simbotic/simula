use std::{collections::hash_map::DefaultHasher, hash::{Hash, Hasher}};

use bevy::{prelude::*, render::view::NoFrustumCulling};
use bevy_inspector_egui::WorldInspectorPlugin;
use rand::{SeedableRng, Rng};
use simula_hexgrid::{
    worldmap::*,
    mapcam::*,
};
use rand_chacha::ChaCha8Rng;


fn main() {
    App::new()
        .insert_resource(WindowDescriptor {
            title: "[Simbotic] Simula - Hexgrid".to_string(),
            width: 940.,
            height: 528.,
            ..Default::default()
        })
        .insert_resource(Msaa { samples: 4 })
        .insert_resource(ClearColor(Color::rgb(0.125, 0.12, 0.13)))
        .add_plugins(DefaultPlugins)
        .add_plugin(WorldInspectorPlugin::new())
        .add_plugin(WorldmapPlugin)
        .add_startup_system(worldmap_setup)
        .add_startup_system(hexagon_builder)
        .run();
}

pub fn worldmap_setup(
    mut commands: Commands,
    assets: Res<AssetServer>,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
) {
    commands
        .spawn()
        .insert_bundle((
            meshes.add(Mesh::from(shape::Capsule {
                depth: 0.5,
                latitudes: 4,
                longitudes: 6,
                ..Default::default()
            })),
            Transform::from_xyz(10.0, 0.0, -10.0),
            GlobalTransform::default(),
            InstanceMaterialData(
                (-51..77)
                    .flat_map(|x| (-51..77).map(move |z| (x as f32 / 10.0, z as f32 / 10.0)))
                    .map(|(x, z)| InstanceData {
                        position: Vec3::new(
                            x * -10.0 + 2.0,
                            0.0,
                            z * 10.0 + (0.5 * ((x * 10.0) % 2.0)),
                        ),
                        scale: 1.3,
                        color: Color::hsla(238.0, 0.95, 0.59, 0.0).as_rgba_u32(),
                    })
                    .collect(),
            ),
            Visibility::default(),
            ComputedVisibility::default(),
            NoFrustumCulling,
        ))
        .insert(WorldMapObject)
        .insert(HexagonTiles);

    let worldmap_plane_material = StandardMaterial {
        emissive_texture: Some(assets.load("textures/worldmap_sal.basis")),
        emissive: Color::WHITE,
        ..Default::default()
    };

    commands
        .spawn()
        .insert_bundle(PbrBundle {
            mesh: meshes.add(Mesh::from(shape::Plane { size: 5000.0 })),
            material: materials.add(worldmap_plane_material),
            transform: Transform::from_xyz(-800.0, 0.0, 800.0),
            ..Default::default()
        })
        .insert(WorldMapObject);

    // camera
    commands
        .spawn_bundle(PerspectiveCameraBundle {
            transform: Transform::from_xyz(0.0, 0.0, 0.0).looking_at(Vec3::ZERO, Vec3::Y),
            ..default()
        })
        .insert(OrbitCamera {
            pan_sensitivity: 10.0,
            center: Vec3::ZERO,
            ..Default::default()
        })
        .insert(WorldMapObject);
}

pub fn hexagon_builder(mut shortest_path: ResMut<ShortestPathBuilder>) {
    shortest_path.counter_one = 0;

    // Loop while `counter` is less than 2048
    while shortest_path.counter_one < 2048 {
        shortest_path.counter_two = 0;

        while shortest_path.counter_two < 2048 {
            let n = shortest_path.counter_one;
            let m = shortest_path.counter_two;

            //hash from vec to make seed for deterministic random complexity value
            let vec = vec![n, m];
            let mut hash = DefaultHasher::new();
            vec.hash(&mut hash);
            let complexity_seed = hash.finish();
            shortest_path.random_complexity =
                ChaCha8Rng::seed_from_u64(complexity_seed).gen_range(0.0..20.0);
            let l = shortest_path.random_complexity;

            //create nodes
            shortest_path.nodes.insert((n, m), l);

            //increment
            shortest_path.counter_two += 1;
        }
        //increment
        shortest_path.counter_one += 1;
    }
}
