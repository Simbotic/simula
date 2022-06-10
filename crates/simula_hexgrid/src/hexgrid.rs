use crate::pathfinding::*;
use simula_camera::orbitcam::*;
use bevy::{
    core_pipeline::Transparent3d,
    ecs::system::{lifetimeless::*, SystemParamItem},
    math::prelude::*,
    pbr::{MeshPipeline, MeshPipelineKey, MeshUniform, SetMeshBindGroup, SetMeshViewBindGroup},
    prelude::*,
    render::{
        mesh::{GpuBufferInfo, MeshVertexBufferLayout},
        render_asset::RenderAssets,
        render_component::{ExtractComponent, ExtractComponentPlugin},
        render_phase::{
            AddRenderCommand, DrawFunctions, EntityRenderCommand, RenderCommandResult, RenderPhase,
            SetItemPipeline, TrackedRenderPass,
        },
        render_resource::*,
        renderer::RenderDevice,
        view::{ComputedVisibility, ExtractedView, Msaa, NoFrustumCulling, Visibility},
        RenderApp, RenderStage,
    },
};
use bevy_egui::*;
use bytemuck::{Pod, Zeroable};
use rand::prelude::*;
use simula_core::prng::*;

#[derive(Component)]
pub struct HexgridObject;

#[derive(Component)]
pub struct TempHexTiles;

#[derive(Component)]
pub struct HexagonTiles;

use std::hash::Hash;
use std::{
    collections::{hash_map::DefaultHasher, HashMap},
    hash::Hasher,
};

pub struct DespawnTileEvent;
pub struct CalculatePathEvent;

pub struct RenderPathEvent {
    value: RenderAction,
}

enum RenderAction {
    RenderUp,
    RenderDown,
    RenderLeft,
    RenderRight,
    Rerender,
}

pub struct NodeStartEnd {
    pub startx: i32,
    pub starty: i32,
    pub endx: i32,
    pub endy: i32,
    pub queue_end: (i32, i32),
    pub destination_reached: bool,
    pub queue_node: Vec<((i32, i32), f32, Vec<(i32, i32)>, f32)>,
    pub nodes_weighted: HashMap<(i32, i32), (f32, f32)>,
    pub node_astar_scores: HashMap<(i32, i32), f32>,
    pub start_weight: f32,
}

impl Default for NodeStartEnd {
    fn default() -> Self {
        NodeStartEnd {
            startx: 1,
            starty: 2,
            endx: 2,
            endy: 3,
            queue_end: (2, 3),
            destination_reached: true,
            queue_node: vec![((0, 0), 0.0, Vec::<(i32, i32)>::new(), 0.0)],
            nodes_weighted: HashMap::new(),
            node_astar_scores: HashMap::new(),
            start_weight: 0.0,
        }
    }
}

pub struct ShortestPathBuilder {
    pub render_min_column: i32,
    pub render_max_column: i32,
    pub render_min_row: i32,
    pub render_max_row: i32,
    pub render_size: i32,
    pub nodes: HashMap<(i32, i32), f32>,
    pub min_column: i32,
    pub max_column: i32,
    pub min_row: i32,
    pub max_row: i32,
    pub tile_coord_x: i32,
    pub tile_coord_z: i32,
    pub counter_one: i32,
    pub counter_two: i32,
    pub shortest_highlight: Vec<(i32, i32)>,
    pub random_complexity: f32,
}

impl Default for ShortestPathBuilder {
    fn default() -> Self {
        ShortestPathBuilder {
            render_min_column: -51,
            render_max_column: 77,
            render_min_row: -51,
            render_max_row: 77,
            render_size: 128,
            nodes: HashMap::new(),
            min_column: -1,
            max_column: 2048,
            min_row: -1,
            max_row: 2048,
            tile_coord_x: 0,
            tile_coord_z: 0,
            counter_one: 0,
            counter_two: 0,
            shortest_highlight: vec![(1, 2), (2, 3)],
            random_complexity: 0.0,
        }
    }
}

pub fn hexgrid_viewer(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut shortest_path: ResMut<ShortestPathBuilder>,
    mut render_path_event: EventReader<RenderPathEvent>,
    mut orbit_camera: Query<&mut OrbitCamera>,
    mut despawn_tile_event: EventWriter<DespawnTileEvent>,
) {
    for mut orbit_camera in orbit_camera.iter_mut() {
        for event in render_path_event.iter() {
            match &event.value {
                RenderAction::RenderUp => {
                    if shortest_path.render_max_row < 2048 {
                        shortest_path.render_min_row += shortest_path.render_size;
                        shortest_path.render_max_row += shortest_path.render_size;
                    }
                }
                RenderAction::RenderDown => {
                    if shortest_path.render_min_row >= 0 {
                        shortest_path.render_min_row -= shortest_path.render_size;
                        shortest_path.render_max_row -= shortest_path.render_size;
                    }
                }
                RenderAction::RenderLeft => {
                    if shortest_path.render_min_column >= 0 {
                        shortest_path.render_min_column -= shortest_path.render_size;
                        shortest_path.render_max_column -= shortest_path.render_size;
                    }
                }
                RenderAction::RenderRight => {
                    if shortest_path.render_max_column < 2048 {
                        shortest_path.render_min_column += shortest_path.render_size;
                        shortest_path.render_max_column += shortest_path.render_size;
                    }
                }

                _ => {}
            }
            orbit_camera.center.z =
                (shortest_path.render_min_row + shortest_path.render_size * 2 / 5) as f32;
            orbit_camera.center.x =
                -(shortest_path.render_min_column + shortest_path.render_size * 2 / 5) as f32;

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
                        (shortest_path.render_min_column..shortest_path.render_max_column)
                            .flat_map(|x| {
                                (shortest_path.render_min_row..shortest_path.render_max_row)
                                    .map(move |z| (x as f32 / 10.0, z as f32 / 10.0))
                            })
                            .map(|(x, z)| HexData {
                                position: Vec3::new(
                                    x * -10.0 + 2.0,
                                    0.0,
                                    z * 10.0 + (0.5 * ((x * 10.0) % 2.0)),
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

            despawn_tile_event.send(DespawnTileEvent);
        }
    }
}

pub fn hexgrid_rebuilder(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut shortest_path: ResMut<ShortestPathBuilder>,
    mouse_button_input: Res<Input<MouseButton>>,
    orbit_camera: Query<&mut OrbitCamera>,
    mut despawn_tile_event: EventWriter<DespawnTileEvent>,
) {
    for orbit_camera in orbit_camera.iter() {
        if mouse_button_input.pressed(MouseButton::Right) {
            shortest_path.render_min_row =
                orbit_camera.center.z as i32 - shortest_path.render_size * 2 / 5;
            shortest_path.render_max_row = shortest_path.render_min_row + shortest_path.render_size;
            shortest_path.render_min_column =
                -shortest_path.render_size * 2 / 5 - orbit_camera.center.x as i32;
            shortest_path.render_max_column =
                shortest_path.render_min_column + shortest_path.render_size;

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
                        (shortest_path.render_min_column..shortest_path.render_max_column)
                            .flat_map(|x| {
                                (shortest_path.render_min_row..shortest_path.render_max_row)
                                    .map(move |z| (x as f32 / 10.0, z as f32 / 10.0))
                            })
                            .map(|(x, z)| HexData {
                                position: Vec3::new(
                                    x * -10.0 + 2.0,
                                    0.0,
                                    z * 10.0 + (0.5 * ((x * 10.0) % 2.0)),
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

            despawn_tile_event.send(DespawnTileEvent);
        }
    }
}

pub fn select_tile(
    mut maps: Query<&mut HexgridData>,
    shortest_path: ResMut<ShortestPathBuilder>,
    mut despawn_tile_event: EventReader<DespawnTileEvent>,
    mut tile_visibility: Query<&mut Visibility, With<TempHexTiles>>,
    mut commands: Commands,
    despawn_tile_objects: Query<Entity, (With<HexagonTiles>, Without<TempHexTiles>)>,
    hex_tile_objects: Query<Entity, With<TempHexTiles>>,
) {
    for mut map in maps.iter_mut() {
        if map.len() == shortest_path.render_size as usize * shortest_path.render_size as usize {
            (shortest_path.render_min_column..shortest_path.render_max_column)
                .flat_map(|x| {
                    (shortest_path.render_min_row..shortest_path.render_max_row)
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
            (shortest_path.render_min_column..shortest_path.render_max_column)
                .flat_map(|x| {
                    (shortest_path.render_min_row..shortest_path.render_max_row)
                        .map(move |z| (x, z))
                })
                .enumerate()
                .filter(|&(_i, x)| shortest_path.shortest_highlight.contains(&x))
                .for_each(|(i, _x)| {
                    map.0[i].color = Color::hsla(360.0, 1.0, 0.5, 0.1).as_rgba_u32()
                });

            for _event in despawn_tile_event.iter() {
                for mut visibility in tile_visibility.iter_mut() {
                    for ent_despawn in despawn_tile_objects.iter() {
                        for ent in hex_tile_objects.iter() {
                            visibility.is_visible = true;
                            commands
                                .entity(ent)
                                .insert(HexagonTiles)
                                .remove::<TempHexTiles>();
                            commands.entity(ent_despawn).despawn_recursive();
                        }
                    }
                }
            }
        }
    }
}

#[derive(Component, Deref)]
pub struct HexgridData(pub Vec<HexData>);
impl ExtractComponent for HexgridData {
    type Query = &'static HexgridData;
    type Filter = ();

    fn extract_component(item: bevy::ecs::query::QueryItem<Self::Query>) -> Self {
        HexgridData(item.0.clone())
    }
}

pub struct HexgridMaterialPlugin;

impl Plugin for HexgridMaterialPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugin(ExtractComponentPlugin::<HexgridData>::default());
        app.sub_app_mut(RenderApp)
            .add_render_command::<Transparent3d, DrawCustom>()
            .init_resource::<HexgridPipeline>()
            .init_resource::<SpecializedMeshPipelines<HexgridPipeline>>()
            .add_system_to_stage(RenderStage::Queue, queue_custom)
            .add_system_to_stage(RenderStage::Prepare, prepare_instance_buffers);
    }
}

#[derive(Clone, Copy, Pod, Zeroable)]
#[repr(C)]
pub struct HexData {
    pub position: Vec3,
    pub scale: f32,
    pub color: u32,
}

#[allow(clippy::too_many_arguments)]
fn queue_custom(
    transparent_3d_draw_functions: Res<DrawFunctions<Transparent3d>>,
    custom_pipeline: Res<HexgridPipeline>,
    msaa: Res<Msaa>,
    mut pipelines: ResMut<SpecializedMeshPipelines<HexgridPipeline>>,
    mut pipeline_cache: ResMut<PipelineCache>,
    meshes: Res<RenderAssets<Mesh>>,
    material_meshes: Query<
        (Entity, &MeshUniform, &Handle<Mesh>),
        (With<Handle<Mesh>>, With<HexgridData>),
    >,
    mut views: Query<(&ExtractedView, &mut RenderPhase<Transparent3d>)>,
) {
    let draw_custom = transparent_3d_draw_functions
        .read()
        .get_id::<DrawCustom>()
        .unwrap();

    let msaa_key = MeshPipelineKey::from_msaa_samples(msaa.samples);

    for (view, mut transparent_phase) in views.iter_mut() {
        let view_matrix = view.transform.compute_matrix();
        let view_row_2 = view_matrix.row(2);
        for (entity, mesh_uniform, mesh_handle) in material_meshes.iter() {
            if let Some(mesh) = meshes.get(mesh_handle) {
                let key =
                    msaa_key | MeshPipelineKey::from_primitive_topology(mesh.primitive_topology);
                let pipeline = pipelines
                    .specialize(&mut pipeline_cache, &custom_pipeline, key, &mesh.layout)
                    .unwrap();
                transparent_phase.add(Transparent3d {
                    entity,
                    pipeline,
                    draw_function: draw_custom,
                    distance: view_row_2.dot(mesh_uniform.transform.col(3)),
                });
            }
        }
    }
}

#[derive(Component)]
pub struct InstanceBuffer {
    buffer: Buffer,
    length: usize,
}

fn prepare_instance_buffers(
    mut commands: Commands,
    query: Query<(Entity, &HexgridData)>,
    render_device: Res<RenderDevice>,
) {
    for (entity, instance_data) in query.iter() {
        let buffer = render_device.create_buffer_with_data(&BufferInitDescriptor {
            label: Some("instance data buffer"),
            contents: bytemuck::cast_slice(instance_data.as_slice()),
            usage: BufferUsages::VERTEX | BufferUsages::COPY_DST,
        });
        commands.entity(entity).insert(InstanceBuffer {
            buffer,
            length: instance_data.len(),
        });
    }
}

pub struct HexgridPipeline {
    shader: Handle<Shader>,
    mesh_pipeline: MeshPipeline,
}

impl FromWorld for HexgridPipeline {
    fn from_world(world: &mut World) -> Self {
        let world = world.cell();
        let asset_server = world.get_resource::<AssetServer>().unwrap();
        asset_server.watch_for_changes().unwrap();
        let shader = asset_server.load("shaders/hexgrid.wgsl");

        let mesh_pipeline = world.get_resource::<MeshPipeline>().unwrap();

        HexgridPipeline {
            shader,
            mesh_pipeline: mesh_pipeline.clone(),
        }
    }
}

impl SpecializedMeshPipeline for HexgridPipeline {
    type Key = MeshPipelineKey;

    fn specialize(
        &self,
        key: Self::Key,
        layout: &MeshVertexBufferLayout,
    ) -> Result<RenderPipelineDescriptor, SpecializedMeshPipelineError> {
        let mut descriptor = self.mesh_pipeline.specialize(key, layout)?;
        descriptor.vertex.shader = self.shader.clone();
        descriptor.vertex.buffers.push(VertexBufferLayout {
            array_stride: std::mem::size_of::<HexData>() as u64,
            step_mode: VertexStepMode::Instance,
            attributes: vec![
                VertexAttribute {
                    format: VertexFormat::Float32x4,
                    offset: 0,
                    shader_location: 3, // shader locations 0-2 are taken up by Position, Normal and UV attributes
                },
                VertexAttribute {
                    format: VertexFormat::Uint32,
                    offset: VertexFormat::Float32x4.size(),
                    shader_location: 4,
                },
            ],
        });
        descriptor.fragment.as_mut().unwrap().shader = self.shader.clone();
        descriptor.layout = Some(vec![
            self.mesh_pipeline.view_layout.clone(),
            self.mesh_pipeline.mesh_layout.clone(),
        ]);

        Ok(descriptor)
    }
}

type DrawCustom = (
    SetItemPipeline,
    SetMeshViewBindGroup<0>,
    SetMeshBindGroup<1>,
    DrawMeshInstanced,
);

pub struct DrawMeshInstanced;
impl EntityRenderCommand for DrawMeshInstanced {
    type Param = (
        SRes<RenderAssets<Mesh>>,
        SQuery<Read<Handle<Mesh>>>,
        SQuery<Read<InstanceBuffer>>,
    );
    #[inline]
    fn render<'w>(
        _view: Entity,
        item: Entity,
        (meshes, mesh_query, instance_buffer_query): SystemParamItem<'w, '_, Self::Param>,
        pass: &mut TrackedRenderPass<'w>,
    ) -> RenderCommandResult {
        let mesh_handle = mesh_query.get(item).unwrap();
        let instance_buffer = instance_buffer_query.get_inner(item).unwrap();

        let gpu_mesh = match meshes.into_inner().get(mesh_handle) {
            Some(gpu_mesh) => gpu_mesh,
            None => return RenderCommandResult::Failure,
        };

        pass.set_vertex_buffer(0, gpu_mesh.vertex_buffer.slice(..));
        pass.set_vertex_buffer(1, instance_buffer.buffer.slice(..));

        match &gpu_mesh.buffer_info {
            GpuBufferInfo::Indexed {
                buffer,
                index_format,
                count,
            } => {
                pass.set_index_buffer(buffer.slice(..), 0, *index_format);
                pass.draw_indexed(0..*count, 0, 0..instance_buffer.length as u32);
            }
            GpuBufferInfo::NonIndexed { vertex_count } => {
                pass.draw(0..*vertex_count, 0..instance_buffer.length as u32);
            }
        }
        RenderCommandResult::Success
    }
}

pub fn ui_system_pathfinding_window(
    mut egui_ctx: ResMut<EguiContext>,
    mut node_start_end: ResMut<NodeStartEnd>,
    mut shortest_path: ResMut<ShortestPathBuilder>,
    mut calculate_path_event: EventWriter<CalculatePathEvent>,
) {
    let end_node = (node_start_end.endx, node_start_end.endy);

    if node_start_end.queue_end != end_node {
        if node_start_end.destination_reached == false {
            calculate_path_event.send(CalculatePathEvent);
        }
    } else {
        node_start_end.destination_reached = true;
    }
    pathfinding_window(
        &mut egui_ctx,
        &mut node_start_end,
        &mut shortest_path,
        calculate_path_event,
    );
}

fn pathfinding_window(
    egui_ctx: &mut ResMut<EguiContext>,
    node_start_end: &mut ResMut<NodeStartEnd>,
    shortest_path: &mut ResMut<ShortestPathBuilder>,
    mut calculate_path_event: EventWriter<CalculatePathEvent>,
) {
    egui::Window::new("Pathfinding")
        .default_width(200.0)
        .anchor(egui::Align2::RIGHT_TOP, egui::Vec2::new(-25.0, 150.0))
        .resizable(false)
        .vscroll(true)
        .show(egui_ctx.ctx_mut(), |ui| {
            ui.group(|ui| {
                ui.horizontal(|ui| {
                    ui.label("Start: ");
                    ui.add(
                        egui::DragValue::new(&mut node_start_end.startx)
                            .clamp_range::<i32>(0..=2048),
                    );
                    ui.add(
                        egui::DragValue::new(&mut node_start_end.starty)
                            .clamp_range::<i32>(0..=2048),
                    );
                    if ui.button("Random").clicked() {
                        node_start_end.startx = rand::thread_rng().gen_range(0..=2048) as i32;
                        node_start_end.starty = rand::thread_rng().gen_range(0..=2048) as i32;
                    }
                });
                ui.horizontal(|ui| {
                    ui.label("End: ");
                    ui.add(
                        egui::DragValue::new(&mut node_start_end.endx).clamp_range::<i32>(0..=2048),
                    );
                    ui.add(
                        egui::DragValue::new(&mut node_start_end.endy).clamp_range::<i32>(0..=2048),
                    );
                    if ui.button("Random").clicked() {
                        node_start_end.endx = rand::thread_rng().gen_range(0..=2048) as i32;
                        node_start_end.endy = rand::thread_rng().gen_range(0..=2048) as i32;
                    }
                });
                ui.horizontal(|ui| {
                    if ui.button("Find Best Path").clicked() {
                        calculate_path_event.send(CalculatePathEvent);
                        node_start_end.destination_reached = true;
                    }
                });
                ui.horizontal(|ui| {
                    ui.label("Shortest Path: ");
                    if node_start_end.destination_reached == true {
                        ui.add(
                            egui::Label::new(format!("{:?}", shortest_path.shortest_highlight))
                                .wrap(true),
                        );
                    } else {
                        ui.add(egui::Label::new(format!("Finding Path...")));
                    }
                });
            });
        });
}

pub fn ui_render_next_tiles(
    mut egui_ctx: ResMut<EguiContext>,
    render_path_event: EventWriter<RenderPathEvent>,
    shortest_path: ResMut<ShortestPathBuilder>,
) {
    render_next_tiles(&mut egui_ctx, render_path_event, shortest_path);
}

fn render_next_tiles(
    egui_ctx: &mut ResMut<EguiContext>,
    mut render_path_event: EventWriter<RenderPathEvent>,
    mut shortest_path: ResMut<ShortestPathBuilder>,
) {
    egui::Window::new("Render Next Tiles")
        .default_width(200.0)
        .anchor(egui::Align2::CENTER_BOTTOM, egui::Vec2::new(0.0, -50.0))
        .resizable(true)
        .vscroll(true)
        .show(egui_ctx.ctx_mut(), |ui| {
            if ui.button("UP").clicked() {
                render_path_event.send(RenderPathEvent {
                    value: RenderAction::RenderUp,
                });
            }
            if ui.button("DOWN").clicked() {
                render_path_event.send(RenderPathEvent {
                    value: RenderAction::RenderDown,
                });
            }
            if ui.button("LEFT").clicked() {
                render_path_event.send(RenderPathEvent {
                    value: RenderAction::RenderLeft,
                });
            }
            if ui.button("RIGHT").clicked() {
                render_path_event.send(RenderPathEvent {
                    value: RenderAction::RenderRight,
                });
            }
            ui.horizontal(|ui| {
                ui.label(format!(
                    "Columns: {} to {}; Rows: {} to {}",
                    shortest_path.render_min_column,
                    shortest_path.render_max_column,
                    shortest_path.render_min_row,
                    shortest_path.render_max_row
                ));
            });
            ui.horizontal(|ui| {
                if ui.button("64").clicked() {
                    shortest_path.render_min_row = 0;
                    shortest_path.render_max_row = 63;
                    shortest_path.render_min_column = 0;
                    shortest_path.render_max_column = 63;
                    shortest_path.render_size = 63;
                    render_path_event.send(RenderPathEvent {
                        value: RenderAction::Rerender,
                    });
                }
                if ui.button("128").clicked() {
                    shortest_path.render_min_row = 0;
                    shortest_path.render_max_row = 127;
                    shortest_path.render_min_column = 0;
                    shortest_path.render_max_column = 127;
                    shortest_path.render_size = 127;
                    render_path_event.send(RenderPathEvent {
                        value: RenderAction::Rerender,
                    });
                }
                if ui.button("256").clicked() {
                    shortest_path.render_min_row = 0;
                    shortest_path.render_max_row = 255;
                    shortest_path.render_min_column = 0;
                    shortest_path.render_max_column = 255;
                    shortest_path.render_size = 255;
                    render_path_event.send(RenderPathEvent {
                        value: RenderAction::Rerender,
                    });
                }
                if ui.button("512").clicked() {
                    shortest_path.render_min_row = 0;
                    shortest_path.render_max_row = 511;
                    shortest_path.render_min_column = 0;
                    shortest_path.render_max_column = 511;
                    shortest_path.render_size = 511;
                    render_path_event.send(RenderPathEvent {
                        value: RenderAction::Rerender,
                    });
                }
            });
            ui.horizontal(|ui| {
                ui.label("Go to tile:".to_string());
                ui.add(
                    egui::DragValue::new(&mut shortest_path.tile_coord_z)
                        .clamp_range::<i32>(0..=2048),
                );
                ui.add(
                    egui::DragValue::new(&mut shortest_path.tile_coord_x)
                        .clamp_range::<i32>(0..=2048),
                );
                if ui.button("Enter").clicked() {
                    shortest_path.render_min_row =
                        shortest_path.tile_coord_x - shortest_path.render_size / 2;
                    shortest_path.render_max_row =
                        shortest_path.render_min_row + shortest_path.render_size;
                    shortest_path.render_min_column =
                        shortest_path.tile_coord_z - shortest_path.render_size / 2;
                    shortest_path.render_max_column =
                        shortest_path.render_min_column + shortest_path.render_size;
                    render_path_event.send(RenderPathEvent {
                        value: RenderAction::Rerender,
                    });
                }
                if ui.button("Random").clicked() {
                    shortest_path.tile_coord_x = rand::thread_rng().gen_range(0..=2048) as i32;
                    shortest_path.tile_coord_z = rand::thread_rng().gen_range(0..=2048) as i32;
                }
            });
        });
}

pub fn hexagon_pathfinder(
    mut node_start_end: ResMut<NodeStartEnd>,
    mut shortest_path: ResMut<ShortestPathBuilder>,
    mut calculate_path_event: EventReader<CalculatePathEvent>,
) {
    for _event in calculate_path_event.iter() {
        // you are here
        let start_node = (node_start_end.startx, node_start_end.starty);

        // you want to go here
        let end_node = (node_start_end.endx, node_start_end.endy);

        // the hexagon arrangement you are using
        let orientation = HexOrientation::FlatTopOddUp;

        if node_start_end.destination_reached == true {
            node_start_end.nodes_weighted = HashMap::new();
            // calculate a weighting for each node based on its distance from the end node
            for (k, v) in shortest_path.nodes.iter() {
                node_start_end.nodes_weighted.insert(
                    k.to_owned(),
                    (
                        v.to_owned(),
                        calculate_node_weight(k, &end_node, &orientation),
                    ),
                );
            }
            node_start_end.start_weight = match node_start_end.nodes_weighted.get(&start_node) {
                Some(x) => x.1,
                None => panic!("Unable to find node weight"),
            };
            let start_weight = node_start_end.start_weight;
            // every time we process a new node we add it to a map
            // if a node has already been recorded then we replace it if it has a better a-star score (smaller number)
            // otherwise we discard it.
            // this is used to optimise the searching whereby if we find a new path to a previously
            // discovered node we can quickly decide to discard or explore the new route
            node_start_end.node_astar_scores = HashMap::new();
            // add starting node a-star score to data set (starting node score is just its weight)
            node_start_end
                .node_astar_scores
                .insert(start_node, start_weight);

            node_start_end.queue_node = vec![(
                start_node,
                node_start_end.start_weight.clone(), // we haven't moved so starting node score is just its weight
                Vec::<(i32, i32)>::new(),
                0.0,
            )];
        }

        let mut counter = 0;

        // target node will eventually be shifted to first of queue so finish processing once it arrives, meaning that we know the best path
        while node_start_end.queue_node[0].0 != end_node {
            counter += 1;

            // remove the first element ready for processing
            let current_path = node_start_end.queue_node.swap_remove(0);
            // expand the node in the current path
            let available_nodes = node_neighbours_offset(
                current_path.0,
                &orientation,
                shortest_path.min_column,
                shortest_path.max_column,
                shortest_path.min_row,
                shortest_path.max_row,
            );

            // process each new path
            for n in available_nodes.iter() {
                let previous_complexities: f32 = current_path.3; // complexity

                // grab the half complexity of the currrent node
                let current_node_complexity: f32 = (node_start_end
                    .nodes_weighted
                    .get(&current_path.0)
                    .unwrap()
                    .0)
                    * 0.5;

                // grab half the complexity of the neighbour node
                let target_node_complexity: f32 =
                    (node_start_end.nodes_weighted.get(n).unwrap().0) * 0.5;

                // calculate its fields
                let complexity =
                    previous_complexities + target_node_complexity + current_node_complexity;
                let target_weight: f32 = node_start_end.nodes_weighted.get(n).unwrap().1;

                let astar = a_star_score(complexity, target_weight);
                let mut previous_nodes_traversed = current_path.2.clone(); // traversed nodes
                previous_nodes_traversed.push(current_path.0);
                //update the a-star data set
                if node_start_end.node_astar_scores.contains_key(n) {
                    if node_start_end.node_astar_scores.get(n) >= Some(&astar) {
                        // data set contains a worse score so update the set with the better score
                        node_start_end.node_astar_scores.insert(*n, astar);
                        // search the queue to see if we already have a route to this node.
                        // If we do but this new path is better then replace it, otherwise discard
                        let mut new_queue_item_required_for_node = true;
                        for mut q in node_start_end.queue_node.iter_mut() {
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
                            node_start_end.queue_node.push((
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
                    node_start_end.node_astar_scores.insert(*n, astar);
                    // update the queue to process through
                    node_start_end.queue_node.push((
                        *n,
                        astar,
                        previous_nodes_traversed,
                        complexity,
                    ));
                }
            }

            // sort the queue by a-star sores so each loop processes the best
            node_start_end
                .queue_node
                .sort_by(|a, b| a.1.partial_cmp(&b.1).unwrap());

            node_start_end.queue_end = node_start_end.queue_node[0].0.clone();

            if counter >= 3000 {
                break;
            }
            node_start_end.destination_reached = false;
        }
        let mut best_path = node_start_end.queue_node[0].2.clone();
        // add end node to data
        best_path.push(end_node);
        let best = best_path;

        shortest_path.shortest_highlight = best.clone(); 
    }
}

pub struct HexgridPlugin;

impl Plugin for HexgridPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugin(OrbitCameraPlugin)
            .add_plugin(HexgridMaterialPlugin)
            .add_event::<DespawnTileEvent>()
            .add_event::<CalculatePathEvent>()
            .add_event::<RenderPathEvent>()
            .insert_resource(NodeStartEnd {
                startx: 1,
                starty: 2,
                endx: 2,
                endy: 3,
                queue_end: (2, 3),
                destination_reached: true,
                queue_node: vec![((0, 0), 0.0, Vec::<(i32, i32)>::new(), 0.0)],
                nodes_weighted: HashMap::new(),
                node_astar_scores: HashMap::new(),
                start_weight: 0.0,
            })
            .insert_resource(ShortestPathBuilder {
                render_min_column: -51,
                render_max_column: 77,
                render_min_row: -51,
                render_max_row: 77,
                render_size: 128,
                nodes: HashMap::new(),
                min_column: -1,
                max_column: 2048,
                min_row: -1,
                max_row: 2048,
                tile_coord_x: 0,
                tile_coord_z: 0,
                counter_one: 0,
                counter_two: 0,
                shortest_highlight: vec![(1, 2), (2, 3)],
                random_complexity: 0.0,
            })
            .add_system(ui_system_pathfinding_window)
            .add_system(ui_render_next_tiles)
            .add_system(select_tile)
            .add_system(hexagon_pathfinder)
            .add_system(hexgrid_viewer)
            .add_system(hexgrid_rebuilder);
    }
}
