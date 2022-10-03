use crate::{pathfinding::{HexOrientation::*, *}, user_interface::*, worldmap_material::*};
use bevy::{
    math::prelude::*,
    prelude::*,
    render::{
        view::{ComputedVisibility, NoFrustumCulling, Visibility}
    }, 
};

use simula_camera::orbitcam::*;
use simula_core::prng::*;
use std::hash::Hash;
use std::{
    collections::{hash_map::DefaultHasher, HashMap},
    hash::Hasher,
};

#[derive(Component)]
pub struct HexgridObject;

#[derive(Component)]
pub struct TempHexTiles;

#[derive(Component)]
pub struct HexagonTiles;

pub struct RenderPathEvent {
    value: RenderAction,
}

pub enum RenderAction {
    RenderUp,
    RenderDown,
    RenderLeft,
    RenderRight,
    Rerender,
}

pub struct PathFind {
    pub startx: i32,
    pub starty: i32,
    pub endx: i32,
    pub endy: i32,
    pub queue_end: (i32, i32),
    pub destination_reached: bool,
    pub queue_node: Vec<((i32, i32), f32, Vec<(i32, i32)>, f32)>,
    pub nodes: HashMap<(i32, i32), f32>,
    pub counter_one: i32,
    pub counter_two: i32,
    pub shortest_highlight: Vec<(i32, i32)>,
    pub random_complexity: f32,
    pub nodes_weighted: HashMap<(i32, i32), (f32, f32)>,
    pub node_astar_scores: HashMap<(i32, i32), f32>,
    pub start_weight: f32,
    pub orientation: HexOrientation,
}

impl Default for PathFind {
    fn default() -> Self {
        PathFind {
            startx: 1,
            starty: 2,
            endx: 2,
            endy: 3,
            queue_end: (2, 3),
            destination_reached: true,
            queue_node: vec![((0, 0), 0.0, Vec::<(i32, i32)>::new(), 0.0)],
            nodes: HashMap::new(),
            counter_one: 0,
            counter_two: 0,
            shortest_highlight: vec![(1, 2), (2, 3)],
            random_complexity: 0.0,
            nodes_weighted: HashMap::new(),
            node_astar_scores: HashMap::new(),
            start_weight: 0.0,
            orientation: FlatTopOddUp,
        }
    }
}

pub struct RenderTile {
    pub render_min_column: i32,
    pub render_max_column: i32,
    pub render_min_row: i32,
    pub render_max_row: i32,
    pub render_size: i32,
    pub min_column: i32,
    pub max_column: i32,
    pub min_row: i32,
    pub max_row: i32,
    pub tile_coord_x: i32,
    pub tile_coord_z: i32,
}

impl Default for RenderTile {
    fn default() -> Self {
        RenderTile {
            render_min_column: -51,
            render_max_column: 77,
            render_min_row: -51,
            render_max_row: 77,
            render_size: 128,
            min_column: -1,
            max_column: 2048,
            min_row: -1,
            max_row: 2048,
            tile_coord_x: 0,
            tile_coord_z: 0,
        }
    }
}

pub fn hexgrid_viewer(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut render_tile: ResMut<RenderTile>,
    mut render_path_event: EventReader<RenderPathEvent>,
    mut orbit_camera: Query<&mut OrbitCamera>,
    mut despawn_tile_objects: Query<Entity, (With<HexagonTiles>, Without<TempHexTiles>)>,
    mut hex_tile_objects: Query<Entity, With<TempHexTiles>>,
    mut tile_visibility: Query<&mut Visibility, With<TempHexTiles>>,
) {
    for mut orbit_camera in orbit_camera.iter_mut() {
        for event in render_path_event.iter() {
            match &event.value {
                RenderAction::RenderUp => {
                    if render_tile.render_max_row < 2048 {
                        render_tile.render_min_row += render_tile.render_size / 2;
                        render_tile.render_max_row += render_tile.render_size / 2;
                    }
                }
                RenderAction::RenderDown => {
                    if render_tile.render_min_row >= 0 {
                        render_tile.render_min_row -= render_tile.render_size / 2;
                        render_tile.render_max_row -= render_tile.render_size / 2;
                    }
                }
                RenderAction::RenderLeft => {
                    if render_tile.render_min_column >= 0 {
                        render_tile.render_min_column -= render_tile.render_size / 2;
                        render_tile.render_max_column -= render_tile.render_size / 2;
                    }
                }
                RenderAction::RenderRight => {
                    if render_tile.render_max_column < 2048 {
                        render_tile.render_min_column += render_tile.render_size / 2;
                        render_tile.render_max_column += render_tile.render_size / 2;
                    }
                }

                _ => {}
            }
            orbit_camera.center.z =
                ((render_tile.render_min_row + render_tile.render_size * 2 / 5) as f32) * 1.1258;
            orbit_camera.center.x =
                -((render_tile.render_min_column + render_tile.render_size * 2 / 5) as f32 * 0.975);

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
                    HexgridData(
                        (render_tile.render_min_column..render_tile.render_max_column)
                            .flat_map(|x| {
                                (render_tile.render_min_row..render_tile.render_max_row)
                                    .map(move |z| (x as f32 / 10.0, z as f32 / 10.0))
                            })
                            .map(|(x, z)| HexData {
                                position: Vec3::new(
                                    x * -10.0 * 0.975,
                                    0.0,
                                    z * 10.0 * 1.1258 + (0.5629 * ((x * 10.0) % 2.0)),
                                ),
                                scale: 1.3,
                                color: Color::hsla(238.0, 0.95, 0.59, 0.1).as_rgba_u32(),
                            })
                            .collect(),
                    ),
                    Visibility { is_visible: false },
                    ComputedVisibility::default(),
                    NoFrustumCulling,
                ))
                .insert(HexgridObject)
                .insert(TempHexTiles);

                despawn_tile(&mut commands, &mut despawn_tile_objects, &mut hex_tile_objects, &mut tile_visibility);
        }
    }
}

pub fn hexgrid_rebuilder(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut render_tile: ResMut<RenderTile>,
    mouse_button_input: Res<Input<MouseButton>>,
    orbit_camera: Query<&mut OrbitCamera>,
    mut despawn_tile_objects: Query<Entity, (With<HexagonTiles>, Without<TempHexTiles>)>,
    mut hex_tile_objects: Query<Entity, With<TempHexTiles>>,
    mut tile_visibility: Query<&mut Visibility, With<TempHexTiles>>,
) {
    for orbit_camera in orbit_camera.iter() {
        if mouse_button_input.pressed(MouseButton::Right) {
            render_tile.render_min_row =
                (orbit_camera.center.z / 1.1258) as i32 - render_tile.render_size * 2 / 5;
            render_tile.render_max_row = render_tile.render_min_row + render_tile.render_size;
            render_tile.render_min_column =
                -render_tile.render_size * 2 / 5 - (orbit_camera.center.x / 0.975) as i32;
            render_tile.render_max_column =
                render_tile.render_min_column + render_tile.render_size;

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
                    HexgridData(
                        (render_tile.render_min_column..render_tile.render_max_column)
                            .flat_map(|x| {
                                (render_tile.render_min_row..render_tile.render_max_row)
                                    .map(move |z| (x as f32 / 10.0, z as f32 / 10.0))
                            })
                            .map(|(x, z)| HexData {
                                position: Vec3::new(
                                    x * -10.0 * 0.975,
                                    0.0,
                                    z * 10.0 * 1.1258 + (0.5629 * ((x * 10.0) % 2.0)),
                                ),
                                scale: 1.3,
                                color: Color::hsla(238.0, 0.95, 0.59, 0.1).as_rgba_u32(),
                            })
                            .collect(),
                    ),
                    Visibility { is_visible: false },
                    ComputedVisibility::default(),
                    NoFrustumCulling,
                ))
                .insert(TempHexTiles)
                .insert(HexgridObject);
        }
        despawn_tile(&mut commands, &mut despawn_tile_objects, &mut hex_tile_objects, &mut tile_visibility);
    }
}

pub fn select_tile(
    mut maps: Query<&mut HexgridData>,
    render_tile: ResMut<RenderTile>,
    path_find: ResMut<PathFind>,
) {
    for mut map in maps.iter_mut() {
        if map.len() == render_tile.render_size as usize * render_tile.render_size as usize {
            (render_tile.render_min_column..render_tile.render_max_column)
                .flat_map(|x| {
                    (render_tile.render_min_row..render_tile.render_max_row)
                        .map(move |z| (x, z))
                })
                .enumerate()
                .for_each(|(i, (x, z))| {
                    //hash from vec to make seed for deterministic random complexity value
                    let vec = vec![x, z];
                    let mut hash = DefaultHasher::new();
                    vec.hash(&mut hash);
                    let complexity_seed = hash.finish();
                    let l = Prng::range_float_range(&mut Prng::new(complexity_seed), 0.0, 20.0);
                    let mut s = 0.95;

                    //lowers saturation of out of bound tiles
                    if z <= 0 {
                        s = 0.8
                    } else if z > 2048 {
                        s = 0.8
                    }
                    if x <= 0 {
                        s = 0.8
                    } else if x > 2048 {
                        s = 0.8
                    }

                    map.0[i].color = Color::hsla(238.0, s, l / 40.0, 0.1).as_rgba_u32();
                    map.0[i].position.y = l / 40.0;
                });

            //highlight bestpath
            (render_tile.render_min_column..render_tile.render_max_column)
                .flat_map(|x| {
                    (render_tile.render_min_row..render_tile.render_max_row)
                        .map(move |z| (x, z))
                })
                .enumerate()
                .filter(|&(_i, x)| path_find.shortest_highlight.contains(&x))
                .for_each(|(i, _x)| {
                    map.0[i].color = Color::hsla(360.0, 1.0, 0.5, 0.1).as_rgba_u32()
                });
        }
    }
}

pub fn despawn_tile(
    commands: &mut Commands,
    despawn_tile_objects: &mut Query<Entity, (With<HexagonTiles>, Without<TempHexTiles>)>,
    hex_tile_objects: &mut Query<Entity, With<TempHexTiles>>,
    tile_visibility: &mut Query<&mut Visibility, With<TempHexTiles>>,
) {
    for mut visibility in tile_visibility.iter_mut() {
        for ent_despawn in despawn_tile_objects.iter_mut() {
            commands.entity(ent_despawn).despawn_recursive();
        }
        for ent in hex_tile_objects.iter() {
            visibility.is_visible = true;
            commands
                .entity(ent)
                .insert(HexagonTiles)
                .remove::<TempHexTiles>();
        }
    }
}

pub fn hexagon_pathfinder(
    path_find: &mut ResMut<PathFind>,
    render_tile: &mut ResMut<RenderTile>,
) {
        // you are here
        let start_node = (path_find.startx, path_find.starty);

        // you want to go here
        let end_node = (path_find.endx, path_find.endy);

        // the hexagon arrangement you are using
        let orientation = path_find.orientation.clone();


        if path_find.destination_reached == true {
            path_find.nodes_weighted = HashMap::new();
            // calculate a weighting for each node based on its distance from the end node
            for (k, v) in path_find.nodes.clone().iter() {
                path_find.nodes_weighted.insert(
                    k.to_owned(),
                    (
                        v.to_owned(),
                        calculate_node_weight(k, &end_node, &orientation),
                    ),
                );
            }
            path_find.start_weight = match path_find.nodes_weighted.get(&start_node) {
                Some(x) => x.1,
                None => panic!("Unable to find node weight"),
            };
            let start_weight = path_find.start_weight;
            // every time we process a new node we add it to a map
            // if a node has already been recorded then we replace it if it has a better a-star score (smaller number)
            // otherwise we discard it.
            // this is used to optimise the searching whereby if we find a new path to a previously
            // discovered node we can quickly decide to discard or explore the new route
            path_find.node_astar_scores = HashMap::new();
            // add starting node a-star score to data set (starting node score is just its weight)
            path_find
                .node_astar_scores
                .insert(start_node, start_weight);

            path_find.queue_node = vec![(
                start_node,
                path_find.start_weight.clone(), // we haven't moved so starting node score is just its weight
                Vec::<(i32, i32)>::new(),
                0.0,
            )];
        }

        let mut counter = 0;

        // target node will eventually be shifted to first of queue so finish processing once it arrives, meaning that we know the best path
        while path_find.queue_node[0].0 != end_node {
            counter += 1;

            // remove the first element ready for processing
            let current_path = path_find.queue_node.swap_remove(0);
            // expand the node in the current path
            let available_nodes = node_neighbours_offset(
                current_path.0,
                &orientation,
                render_tile.min_column,
                render_tile.max_column,
                render_tile.min_row,
                render_tile.max_row,
            );

            // process each new path
            for n in available_nodes.iter() {
                let previous_complexities: f32 = current_path.3; // complexity

                // grab the half complexity of the currrent node
                let current_node_complexity: f32 = (path_find
                    .nodes_weighted
                    .get(&current_path.0)
                    .unwrap()
                    .0)
                    * 0.5;

                // grab half the complexity of the neighbour node
                let target_node_complexity: f32 =
                    (path_find.nodes_weighted.get(n).unwrap().0) * 0.5;

                // calculate its fields
                let complexity =
                    previous_complexities + target_node_complexity + current_node_complexity;
                let target_weight: f32 = path_find.nodes_weighted.get(n).unwrap().1;

                let astar = a_star_score(complexity, target_weight);
                let mut previous_nodes_traversed = current_path.2.clone(); // traversed nodes
                previous_nodes_traversed.push(current_path.0);
                //update the a-star data set
                if path_find.node_astar_scores.contains_key(n) {
                    if path_find.node_astar_scores.get(n) >= Some(&astar) {
                        // data set contains a worse score so update the set with the better score
                        path_find.node_astar_scores.insert(*n, astar);
                        // search the queue to see if we already have a route to this node.
                        // If we do but this new path is better then replace it, otherwise discard
                        let mut new_queue_item_required_for_node = true;
                        for mut q in path_find.queue_node.iter_mut() {
                            if &q.0 == n {
                                // if existing score is worse then replace the queue item and
                                // don't allow a fresh queue item to be added
                                if q.1 >= astar {
                                    new_queue_item_required_for_node = false;
                                    q.1 = astar;
                                    q.2 = previous_nodes_traversed.clone();
                                    q.3 = complexity;
                                }
                            }
                        }
                        // queue doesn't contain a route to this node, as we have now found a better route
                        // update the queue with it so it can be explored
                        if new_queue_item_required_for_node {
                            path_find.queue_node.push((
                                *n,
                                astar,
                                previous_nodes_traversed,
                                complexity,
                            ));
                        }
                    }
                } else {
                    // no record of node and new path required in queue
                    // update the a-star score data
                    path_find.node_astar_scores.insert(*n, astar);
                    // update the queue to process through
                    path_find.queue_node.push((
                        *n,
                        astar,
                        previous_nodes_traversed,
                        complexity,
                    ));
                }
            }

            // sort the queue by a-star sores so each loop processes the best
            path_find
                .queue_node
                .sort_by(|a, b| a.1.partial_cmp(&b.1).unwrap());

            path_find.queue_end = path_find.queue_node[0].0.clone();

            if counter >= 3000 {
                break;
            }
            path_find.destination_reached = false;
        }
        let mut best_path = path_find.queue_node[0].2.clone();
        // add end node to data
        best_path.push(end_node);
        let best = best_path;

        path_find.shortest_highlight = best.clone();
}

pub struct HexgridPlugin;

impl Plugin for HexgridPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugin(OrbitCameraPlugin)
            .add_plugin(HexgridMaterialPlugin)
            .add_event::<RenderPathEvent>()
            .insert_resource(PathFind::default())
            .insert_resource(RenderTile::default())
            .add_system(ui_system_pathfinding_window)
            .add_system(ui_render_next_tiles)
            .add_system(select_tile.after(hexgrid_viewer))
            .add_system(hexgrid_viewer.after(ui_render_next_tiles))
            .add_system(hexgrid_rebuilder.after(select_tile));
    }
}
