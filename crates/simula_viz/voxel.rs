use bevy::{
    core_pipeline::Opaque3d,
    ecs::system::{
        lifetimeless::{Read, SQuery, SRes},
        SystemParamItem,
    },
    pbr::MeshPipelineKey,
    prelude::*,
    render::{
        mesh::{Mesh, Indices},
        primitives::Aabb,
        render_component::{ComponentUniforms, DynamicUniformIndex, UniformComponentPlugin},
        render_phase::{
            AddRenderCommand, DrawFunctions, EntityRenderCommand, RenderCommandResult, RenderPhase,
            SetItemPipeline, TrackedRenderPass,
        },
        render_resource::{
            std140::{AsStd140, Std140},
            *,
        },
        renderer::{RenderDevice, RenderQueue},
        texture::BevyDefault,
        view::{ExtractedView, Msaa, ViewUniform, ViewUniformOffset, ViewUniforms},
        RenderApp, RenderStage,
    },
};

#[derive(Debug, Copy, Clone)]
pub struct Voxel {
    pub position: Vec3,
    pub size: f32,
    pub color: Color,
}

impl Voxel {
    pub fn new(position: Vec3, size: f32, color: Color) -> Voxel {
        Voxel {
            position,
            size,
            color,
        }
    }
}

impl Default for Voxel {
    fn default() -> Self {
        Voxel {
            position: Vec3::ZERO,
            size: 1.0,
            color: Color::rgba(1., 0.1, 1., 1.),
        }
    }
}

impl From<Voxel> for Mesh {
    fn from(voxel: Voxel) -> Self {
        let voxel_box: Box = voxel.into();
        let raw_mesh: RawMesh = voxel_box.into();
        raw_mesh.into()
    }
}

#[derive(Debug, Copy, Clone)]
pub struct Box {
    pub min_x: f32,
    pub max_x: f32,

    pub min_y: f32,
    pub max_y: f32,

    pub min_z: f32,
    pub max_z: f32,

    pub color: Color,
}

impl Box {
    pub fn new(x_length: f32, y_length: f32, z_length: f32, color: Color) -> Box {
        Box {
            max_x: x_length / 2.0,
            min_x: -x_length / 2.0,
            max_y: y_length / 2.0,
            min_y: -y_length / 2.0,
            max_z: z_length / 2.0,
            min_z: -z_length / 2.0,
            color,
        }
    }
}

impl Default for Box {
    fn default() -> Self {
        Box::new(2.0, 1.0, 1.0, Color::rgba(1., 0.1, 1., 1.))
    }
}

impl From<Voxel> for Box {
    fn from(voxel: Voxel) -> Self {
        let mut voxel_box = Box::new(voxel.size, voxel.size, voxel.size, voxel.color);
        voxel_box.min_x += voxel.position.x;
        voxel_box.max_x += voxel.position.x;
        voxel_box.min_y += voxel.position.y;
        voxel_box.max_y += voxel.position.y;
        voxel_box.min_z += voxel.position.z;
        voxel_box.max_z += voxel.position.z;
        voxel_box
    }
}

#[derive(Default)]
pub struct RawMesh {
    positions: Vec<[f32; 3]>,
    normals: Vec<[f32; 3]>,
    colors: Vec<[f32; 4]>,
    indices: Vec<u32>,
}

impl RawMesh {
    pub fn extend(&mut self, other: RawMesh) {
        let offset = self.positions.len() as u32;
        self.positions.extend(other.positions);
        self.normals.extend(other.normals);
        self.colors.extend(other.colors);
        self.indices
            .extend(other.indices.into_iter().map(|i| i + offset));
    }
}

impl From<Box> for RawMesh {
    fn from(sp: Box) -> Self {
        #[rustfmt::skip]
        let vertices = &[
            // front
            ([sp.min_x, sp.min_y, sp.max_z], [0., 0., -1.0], [sp.color.r(), sp.color.g(), sp.color.b(), sp.color.a()]),
            ([sp.max_x, sp.min_y, sp.max_z], [0., 0., -1.0], [sp.color.r(), sp.color.g(), sp.color.b(), sp.color.a()]),
            ([sp.max_x, sp.max_y, sp.max_z], [0., 0., -1.0], [sp.color.r(), sp.color.g(), sp.color.b(), sp.color.a()]),
            ([sp.min_x, sp.max_y, sp.max_z], [0., 0., -1.0], [sp.color.r(), sp.color.g(), sp.color.b(), sp.color.a()]),
            // back
            ([sp.min_x, sp.max_y, sp.min_z], [0., 0., 1.0], [sp.color.r(), sp.color.g(), sp.color.b(), sp.color.a()]),
            ([sp.max_x, sp.max_y, sp.min_z], [0., 0., 1.0], [sp.color.r(), sp.color.g(), sp.color.b(), sp.color.a()]),
            ([sp.max_x, sp.min_y, sp.min_z], [0., 0., 1.0], [sp.color.r(), sp.color.g(), sp.color.b(), sp.color.a()]),
            ([sp.min_x, sp.min_y, sp.min_z], [0., 0., 1.0], [sp.color.r(), sp.color.g(), sp.color.b(), sp.color.a()]),
            // right
            ([sp.max_x, sp.min_y, sp.min_z], [1.0, 0., 0.], [sp.color.r(), sp.color.g(), sp.color.b(), sp.color.a()]),
            ([sp.max_x, sp.max_y, sp.min_z], [1.0, 0., 0.], [sp.color.r(), sp.color.g(), sp.color.b(), sp.color.a()]),
            ([sp.max_x, sp.max_y, sp.max_z], [1.0, 0., 0.], [sp.color.r(), sp.color.g(), sp.color.b(), sp.color.a()]),
            ([sp.max_x, sp.min_y, sp.max_z], [1.0, 0., 0.], [sp.color.r(), sp.color.g(), sp.color.b(), sp.color.a()]),
            // left
            ([sp.min_x, sp.min_y, sp.max_z], [-1.0, 0., 0.], [sp.color.r(), sp.color.g(), sp.color.b(), sp.color.a()]),
            ([sp.min_x, sp.max_y, sp.max_z], [-1.0, 0., 0.], [sp.color.r(), sp.color.g(), sp.color.b(), sp.color.a()]),
            ([sp.min_x, sp.max_y, sp.min_z], [-1.0, 0., 0.], [sp.color.r(), sp.color.g(), sp.color.b(), sp.color.a()]),
            ([sp.min_x, sp.min_y, sp.min_z], [-1.0, 0., 0.], [sp.color.r(), sp.color.g(), sp.color.b(), sp.color.a()]),
            // up
            ([sp.max_x, sp.max_y, sp.min_z], [0., 1.0, 0.], [sp.color.r(), sp.color.g(), sp.color.b(), sp.color.a()]),
            ([sp.min_x, sp.max_y, sp.min_z], [0., 1.0, 0.], [sp.color.r(), sp.color.g(), sp.color.b(), sp.color.a()]),
            ([sp.min_x, sp.max_y, sp.max_z], [0., 1.0, 0.], [sp.color.r(), sp.color.g(), sp.color.b(), sp.color.a()]),
            ([sp.max_x, sp.max_y, sp.max_z], [0., 1.0, 0.], [sp.color.r(), sp.color.g(), sp.color.b(), sp.color.a()]),
            // bottom
            ([sp.max_x, sp.min_y, sp.max_z], [0., -1.0, 0.], [sp.color.r(), sp.color.g(), sp.color.b(), sp.color.a()]),
            ([sp.min_x, sp.min_y, sp.max_z], [0., -1.0, 0.], [sp.color.r(), sp.color.g(), sp.color.b(), sp.color.a()]),
            ([sp.min_x, sp.min_y, sp.min_z], [0., -1.0, 0.], [sp.color.r(), sp.color.g(), sp.color.b(), sp.color.a()]),
            ([sp.max_x, sp.min_y, sp.min_z], [0., -1.0, 0.], [sp.color.r(), sp.color.g(), sp.color.b(), sp.color.a()]),
        ];

        let mut positions = Vec::with_capacity(24);
        let mut normals = Vec::with_capacity(24);
        let mut colors = Vec::with_capacity(24);

        for (position, normal, color) in vertices.iter() {
            positions.push(*position);
            normals.push(*normal);
            colors.push(*color);
        }

        let indices = vec![
            0, 1, 2, 2, 3, 0, // front
            4, 5, 6, 6, 7, 4, // back
            8, 9, 10, 10, 11, 8, // right
            12, 13, 14, 14, 15, 12, // left
            16, 17, 18, 18, 19, 16, // up
            20, 21, 22, 22, 23, 20, // bottom
        ];

        RawMesh {
            positions,
            normals,
            colors,
            indices,
        }
    }
}

// impl mikktspace::Geometry for RodMesh {
//     fn num_faces(&self) -> usize {
//         self.tris.len() / 3
//     }

//     fn num_vertices_of_face(&self, _face: usize) -> usize {
//         3
//     }

//     fn position(&self, face: usize, vert: usize) -> [f32; 3] {
//         let idx = self.tris[face * 3 + vert] as usize;
//         self.vs[idx]
//     }

//     fn normal(&self, face: usize, vert: usize) -> [f32; 3] {
//         let idx = self.tris[face * 3 + vert] as usize;
//         self.vns[idx]
//     }

//     fn tex_coord(&self, face: usize, vert: usize) -> [f32; 2] {
//         let idx = self.tris[face * 3 + vert] as usize;
//         self.vts[idx]
//     }

//     fn set_tangent_encoded(&mut self, tangent: [f32; 4], face: usize, vert: usize) {
//         let idx = self.tris[face * 3 + vert] as usize;
//         self.vxs[idx] = tangent;
//     }
// }

impl From<Voxel> for RawMesh {
    fn from(voxel: Voxel) -> Self {
        let bx: Box = voxel.into();
        bx.into()
    }
}

impl From<RawMesh> for Mesh {
    fn from(raw_mesh: RawMesh) -> Self {
        let mut uvs: Vec<[f32; 2]> = vec![];
        uvs.resize(raw_mesh.positions.len(), [0.0, 0.0]); 
        let mut tangents: Vec<[f32; 4]> = vec![];
        tangents.resize(raw_mesh.positions.len(), [0.0, 0.0, 0.0, 0.0]);
        let mut mesh = Mesh::new(PrimitiveTopology::TriangleList);
        mesh.set_attribute(Mesh::ATTRIBUTE_POSITION, raw_mesh.positions);
        mesh.set_attribute(Mesh::ATTRIBUTE_NORMAL, raw_mesh.normals);
        mesh.set_attribute(Mesh::ATTRIBUTE_UV_0, uvs);
        mesh.set_attribute(Mesh::ATTRIBUTE_TANGENT, tangents);
        mesh.set_attribute(Mesh::ATTRIBUTE_COLOR, raw_mesh.colors);
        mesh.set_indices(Some(Indices::U32(raw_mesh.indices)));
        mesh
    }
}

pub fn merge(voxels: Vec<Voxel>) -> Mesh {
    let mut raw_mesh = RawMesh::default();
    voxels.into_iter().fold(&mut raw_mesh, |raw_mesh, voxel| {
        let voxel_mesh: RawMesh = voxel.into();
        raw_mesh.extend(voxel_mesh);
        raw_mesh
    });
    raw_mesh.into()
}


#[derive(Component, Clone)]
pub struct Voxels {
    pub voxels: Vec<Line>,
}

impl Default for Voxels {
    fn default() -> Self {
        Self { voxels: Vec::new() }
    }
}

#[derive(Bundle)]
pub struct VoxelsBundle {
    pub voxels: Voxels,
    pub material: VoxelsMaterial,
    pub transform: Transform,
    pub global_transform: GlobalTransform,
    pub visibility: Visibility,
    pub computed_visibility: ComputedVisibility,
    pub aabb: Aabb,
}

impl Default for VoxelsBundle {
    fn default() -> Self {
        VoxelsBundle {
            voxels: Voxels::default(),
            material: VoxelsMaterial::default(),
            transform: Transform::default(),
            global_transform: GlobalTransform::default(),
            visibility: Visibility::default(),
            computed_visibility: ComputedVisibility::default(),
            aabb: Aabb {
                center: Vec3::ZERO,
                half_extents: Vec3::ONE,
            },
        }
    }
}

#[derive(Component, Default)]
pub struct VoxelsMaterial;

pub struct VoxelsPlugin;

impl Plugin for VoxelsPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugin(UniformComponentPlugin::<ModelUniform>::default());
        app.sub_app_mut(RenderApp)
            .add_render_command::<Opaque3d, DrawVoxelsCustom>()
            .init_resource::<VoxelsPipeline>()
            .init_resource::<SpecializedPipevoxels<VoxelsPipeline>>()
            .add_system_to_stage(RenderStage::Extract, extract_voxels)
            .add_system_to_stage(RenderStage::Prepare, prepare_voxels)
            .add_system_to_stage(RenderStage::Queue, queue_model_bind_group)
            .add_system_to_stage(RenderStage::Queue, queue_voxels)
            .add_system_to_stage(RenderStage::Queue, queue_view_bind_groups);
    }
}

fn queue_voxels(
    opaque_3d_draw_functions: Res<DrawFunctions<Opaque3d>>,
    voxels_pipeline: Res<VoxelsPipeline>,
    msaa: Res<Msaa>,
    mut pipevoxels: ResMut<SpecializedPipevoxels<VoxelsPipeline>>,
    mut pipeline_cache: ResMut<RenderPipelineCache>,
    material_voxels: Query<(Entity, &ModelUniform), (With<Voxels>, With<VoxelsMaterial>)>,
    mut views: Query<(&ExtractedView, &mut RenderPhase<Opaque3d>)>,
) {
    let draw_voxels = opaque_3d_draw_functions
        .read()
        .get_id::<DrawVoxelsCustom>()
        .unwrap();

    let key = MeshPipelineKey::from_msaa_samples(msaa.samples)
        | MeshPipelineKey::from_primitive_topology(PrimitiveTopology::LineList);
    let pipeline = pipevoxels.specialize(&mut pipeline_cache, &voxels_pipeline, key);

    for (view, mut opaque_phase) in views.iter_mut() {
        trace!("queue_voxels: views.iter_mut()");
        let view_matrix = view.transform.compute_matrix();
        let view_row_2 = view_matrix.row(2);
        for (entity, model_uniform) in material_voxels.iter() {
            trace!("queue_voxels: material_voxels.iter()");
            opaque_phase.add(Opaque3d {
                entity,
                pipeline,
                draw_function: draw_voxels,
                distance: view_row_2.dot(model_uniform.transform.col(3)),
            });
        }
    }
}

#[derive(Component)]
struct ExtractedVoxels {
    num_voxels: usize,
    points: Vec<[f32; 4]>,
    colors: Vec<[f32; 4]>,
}

fn extract_voxels(
    mut previous_len: Local<usize>,
    mut commands: Commands,
    mut voxels: Query<
        (Entity, &mut Voxels, &GlobalTransform, &ComputedVisibility),
        With<VoxelsMaterial>,
    >,
) {
    let mut values = Vec::with_capacity(*previous_len);
    for (entity, mut voxels, transform, visibility) in voxels.iter_mut() {
        if !visibility.is_visible {
            continue;
        }

        let mut points = vec![];
        let mut colors = vec![];

        let mut i = 0;
        let num_voxels = voxels.voxels.len();

        trace!("num_voxels {}", num_voxels);

        points.resize(num_voxels * 2, [0f32; 4]);
        colors.resize(num_voxels * 2, [0f32; 4]);

        for line in voxels.voxels.iter() {
            points[i] = line.start.extend(1.0).into();
            points[i + 1] = line.end.extend(1.0).into();
            colors[i] = line.color[0].as_rgba_f32().into();
            colors[i + 1] = line.color[1].as_rgba_f32().into();
            i += 2;
        }
        voxels.voxels = vec![];

        let transform_matrix = transform.compute_matrix();

        values.push((
            entity,
            (
                voxels.clone(),
                VoxelsMaterial,
                ExtractedVoxels {
                    num_voxels,
                    points,
                    colors,
                },
                ModelUniform {
                    transform: transform_matrix,
                    inverse_transpose_model: transform_matrix.inverse().transpose(),
                    flags: 0,
                },
            ),
        ));
    }

    trace!("extract_voxels {}", values.len());
    *previous_len = values.len();
    commands.insert_or_spawn_batch(values);
}

#[derive(Component)]
struct VoxelsMeta {
    num_voxels: usize,
    points_buffer: Buffer,
    colors_buffer: Buffer,
    bind_group: BindGroup,
}

fn prepare_voxels(
    render_device: Res<RenderDevice>,
    render_queue: Res<RenderQueue>,
    pipeline: Res<VoxelsPipeline>,
    mut commands: Commands,
    voxels: Query<(Entity, &mut ExtractedVoxels), With<VoxelsMaterial>>,
) {
    for (entity, extracted_voxels) in voxels.iter() {
        let points: Vec<u8> = extracted_voxels
            .points
            .iter()
            .flatten()
            .flat_map(|scalar| {
                scalar
                    .as_std140()
                    .as_bytes()
                    .iter()
                    .cloned()
                    .collect::<Vec<u8>>()
            })
            .collect();

        let colors: Vec<u8> = extracted_voxels
            .colors
            .iter()
            .flatten()
            .flat_map(|scalar| {
                scalar
                    .as_std140()
                    .as_bytes()
                    .iter()
                    .cloned()
                    .collect::<Vec<u8>>()
            })
            .collect();

        trace!("points.len() {}", points.len());
        trace!("colors.len() {}", colors.len());

        if points.len() == 0 {
            continue;
        }

        let points_buffer = render_device.create_buffer(&BufferDescriptor {
            label: Some("voxels_points_buffer"),
            size: points.len() as u64,
            usage: BufferUsages::VERTEX | BufferUsages::UNIFORM | BufferUsages::COPY_DST,
            mapped_at_creation: false,
        });

        let colors_buffer = render_device.create_buffer(&BufferDescriptor {
            label: Some("voxels_colors_buffer"),
            size: colors.len() as u64,
            usage: BufferUsages::VERTEX | BufferUsages::UNIFORM | BufferUsages::COPY_DST,
            mapped_at_creation: false,
        });

        render_queue.write_buffer(&points_buffer, 0, bevy::core::cast_slice(&points));
        render_queue.write_buffer(&colors_buffer, 0, bevy::core::cast_slice(&colors));

        let bind_group = render_device.create_bind_group(&BindGroupDescriptor {
            label: Some("voxels_create_bind_group"),
            layout: &pipeline.voxel_bind_group_layout,
            entries: &[
                BindGroupEntry {
                    binding: 0,
                    resource: points_buffer.as_entire_binding(),
                },
                BindGroupEntry {
                    binding: 1,
                    resource: colors_buffer.as_entire_binding(),
                },
            ],
        });

        let voxels_meta = VoxelsMeta {
            num_voxels: extracted_voxels.num_voxels,
            points_buffer,
            colors_buffer,
            bind_group,
        };

        commands.entity(entity).insert(voxels_meta);
    }
}

pub struct VoxelsPipeline {
    shader: Handle<Shader>,
    view_bind_group_layout: BindGroupLayout,
    voxel_bind_group_layout: BindGroupLayout,
    model_bind_group_layout: BindGroupLayout,
}

impl FromWorld for VoxelsPipeline {
    fn from_world(world: &mut World) -> Self {
        let world = world.cell();
        let asset_server = world.get_resource::<AssetServer>().unwrap();
        let shader = asset_server.load("shaders/voxels.wgsl");

        let render_device = world.get_resource_mut::<RenderDevice>().unwrap();

        let view_bind_group_layout =
            render_device.create_bind_group_layout(&BindGroupLayoutDescriptor {
                label: Some("voxels_view_bind_group_layout"),
                entries: &[BindGroupLayoutEntry {
                    binding: 0,
                    visibility: ShaderStages::VERTEX | ShaderStages::FRAGMENT,
                    ty: BindingType::Buffer {
                        ty: BufferBindingType::Uniform,
                        has_dynamic_offset: true,
                        min_binding_size: BufferSize::new(ViewUniform::std140_size_static() as u64),
                    },
                    count: None,
                }],
            });

        let voxel_bind_group_layout =
            render_device.create_bind_group_layout(&BindGroupLayoutDescriptor {
                label: Some("voxels_bind_group_layout"),
                entries: &[
                    BindGroupLayoutEntry {
                        binding: 0,
                        visibility: ShaderStages::VERTEX,
                        ty: BindingType::Buffer {
                            ty: BufferBindingType::Uniform,
                            has_dynamic_offset: false,
                            min_binding_size: BufferSize::new(0),
                        },
                        count: None,
                    },
                    BindGroupLayoutEntry {
                        binding: 1,
                        visibility: ShaderStages::VERTEX,
                        ty: BindingType::Buffer {
                            ty: BufferBindingType::Uniform,
                            has_dynamic_offset: false,
                            min_binding_size: BufferSize::new(0),
                        },
                        count: None,
                    },
                ],
            });

        let model_bind_group_layout = render_device.create_bind_group_layout(&BindGroupLayoutDescriptor {
            label: Some("voxels_model_bind_group_layout"),
            entries: &[BindGroupLayoutEntry {
                binding: 0,
                visibility: ShaderStages::VERTEX | ShaderStages::FRAGMENT,
                ty: BindingType::Buffer {
                    ty: BufferBindingType::Uniform,
                    has_dynamic_offset: true,
                    min_binding_size: BufferSize::new(ModelUniform::std140_size_static() as u64),
                },
                count: None,
            }],
        });

        VoxelsPipeline {
            shader,
            view_bind_group_layout,
            voxel_bind_group_layout,
            model_bind_group_layout,
        }
    }
}

impl SpecializedPipeline for VoxelsPipeline {
    type Key = MeshPipelineKey;

    fn specialize(&self, key: Self::Key) -> RenderPipelineDescriptor {
        let shader_defs = Vec::new();

        RenderPipelineDescriptor {
            vertex: VertexState {
                shader: self.shader.clone(),
                entry_point: "vertex".into(),
                shader_defs: shader_defs.clone(),
                buffers: vec![
                    VertexBufferLayout {
                        array_stride: 16,
                        step_mode: VertexStepMode::Vertex,
                        attributes: vec![VertexAttribute {
                            format: VertexFormat::Float32x4,
                            offset: 0,
                            shader_location: 0,
                        }],
                    },
                    VertexBufferLayout {
                        array_stride: 16,
                        step_mode: VertexStepMode::Vertex,
                        attributes: vec![VertexAttribute {
                            format: VertexFormat::Float32x4,
                            offset: 0,
                            shader_location: 1,
                        }],
                    },
                ],
            },
            fragment: Some(FragmentState {
                shader: self.shader.clone(),
                shader_defs,
                entry_point: "fragment".into(),
                targets: vec![ColorTargetState {
                    format: TextureFormat::bevy_default(),
                    blend: Some(BlendState::REPLACE),
                    write_mask: ColorWrites::ALL,
                }],
            }),
            layout: Some(vec![
                self.view_bind_group_layout.clone(),
                self.voxel_bind_group_layout.clone(),
                self.model_bind_group_layout.clone(),
            ]),
            primitive: PrimitiveState {
                front_face: FrontFace::Ccw,
                cull_mode: Some(Face::Back),
                unclipped_depth: false,
                polygon_mode: PolygonMode::Fill,
                conservative: false,
                topology: key.primitive_topology(),
                strip_index_format: None,
            },
            depth_stencil: Some(DepthStencilState {
                format: TextureFormat::Depth32Float,
                depth_write_enabled: true,
                depth_compare: CompareFunction::Greater,
                stencil: StencilState {
                    front: StencilFaceState::IGNORE,
                    back: StencilFaceState::IGNORE,
                    read_mask: 0,
                    write_mask: 0,
                },
                bias: DepthBiasState {
                    constant: 0,
                    slope_scale: 0.0,
                    clamp: 0.0,
                },
            }),
            multisample: MultisampleState {
                count: key.msaa_samples(),
                mask: !0,
                alpha_to_coverage_enabled: false,
            },
            label: Some("voxels_pipeline".into()),
        }
    }
}

type DrawVoxelsCustom = (
    SetItemPipeline,
    SetVoxelsViewBindGroup<0>,
    SetVoxelsBindGroup<1>,
    SetVoxelsModelBindGroup<2>,
    DrawVoxels,
);

#[derive(Component, AsStd140, Clone)]
pub struct ModelUniform {
    pub transform: Mat4,
    pub inverse_transpose_model: Mat4,
    pub flags: u32,
}

pub struct ModelBindGroup {
    pub value: BindGroup,
}

pub fn queue_model_bind_group(
    mut commands: Commands,
    voxels_pipeline: Res<VoxelsPipeline>,
    render_device: Res<RenderDevice>,
    voxels_uniforms: Res<ComponentUniforms<ModelUniform>>,
) {
    if let Some(binding) = voxels_uniforms.uniforms().binding() {
        trace!("voxels_model_bind_group");
        commands.insert_resource(ModelBindGroup {
            value: render_device.create_bind_group(&BindGroupDescriptor {
                entries: &[BindGroupEntry {
                    binding: 0,
                    resource: binding,
                }],
                label: Some("voxels_model_bind_group"),
                layout: &voxels_pipeline.model_bind_group_layout,
            }),
        });
    }
}

#[derive(Component)]
pub struct VoxelsViewBindGroup {
    pub value: BindGroup,
}

pub fn queue_view_bind_groups(
    mut commands: Commands,
    render_device: Res<RenderDevice>,
    voxels_pipeline: Res<VoxelsPipeline>,
    view_uniforms: Res<ViewUniforms>,
    views: Query<Entity>,
) {
    if let Some(view_binding) = view_uniforms.uniforms.binding() {
        for entity in views.iter() {
            trace!("queue_view_bind_groups: views.iter()");
            let view_bind_group = render_device.create_bind_group(&BindGroupDescriptor {
                entries: &[BindGroupEntry {
                    binding: 0,
                    resource: view_binding.clone(),
                }],
                label: Some("voxels_view_bind_group"),
                layout: &voxels_pipeline.view_bind_group_layout,
            });

            commands.entity(entity).insert(VoxelsViewBindGroup {
                value: view_bind_group,
            });
        }
    }
}

pub struct SetVoxelsViewBindGroup<const I: usize>;
impl<const I: usize> EntityRenderCommand for SetVoxelsViewBindGroup<I> {
    type Param = SQuery<(Read<ViewUniformOffset>, Read<VoxelsViewBindGroup>)>;

    #[inline]
    fn render<'w>(
        view: Entity,
        _item: Entity,
        view_query: SystemParamItem<'w, '_, Self::Param>,
        pass: &mut TrackedRenderPass<'w>,
    ) -> RenderCommandResult {
        if let Ok((view_uniform, voxels_view_bind_group)) = view_query.get(view) {
            trace!("SetVoxelsViewBindGroup: EntityRenderCommand");
            pass.set_bind_group(I, &voxels_view_bind_group.value, &[view_uniform.offset]);
        }

        RenderCommandResult::Success
    }
}

struct SetVoxelsBindGroup<const I: usize>;
impl<const I: usize> EntityRenderCommand for SetVoxelsBindGroup<I> {
    type Param = SQuery<Read<VoxelsMeta>>;

    fn render<'w>(
        _view: Entity,
        item: Entity,
        voxels_meta: SystemParamItem<'w, '_, Self::Param>,
        pass: &mut TrackedRenderPass<'w>,
    ) -> RenderCommandResult {
        if let Ok(voxels_meta) = voxels_meta.get(item) {
            trace!("SetVoxelsBindGroup: EntityRenderCommand");
            pass.set_bind_group(I, &voxels_meta.bind_group, &[]);
        }

        RenderCommandResult::Success
    }
}

pub struct SetVoxelsModelBindGroup<const I: usize>;
impl<const I: usize> EntityRenderCommand for SetVoxelsModelBindGroup<I> {
    type Param = (
        SRes<ModelBindGroup>,
        SQuery<Read<DynamicUniformIndex<ModelUniform>>>,
    );
    #[inline]
    fn render<'w>(
        _view: Entity,
        item: Entity,
        (model_bind_group, model_query): SystemParamItem<'w, '_, Self::Param>,
        pass: &mut TrackedRenderPass<'w>,
    ) -> RenderCommandResult {
        if let Ok(model_index) = model_query.get(item) {
            trace!("SetVoxelsModelBindGroup: EntityRenderCommand");
            pass.set_bind_group(
                I,
                &model_bind_group.into_inner().value,
                &[model_index.index()],
            );
        }
        RenderCommandResult::Success
    }
}

struct DrawVoxels;
impl EntityRenderCommand for DrawVoxels {
    type Param = SQuery<Read<VoxelsMeta>>;

    #[inline]
    fn render<'w>(
        _view: Entity,
        item: Entity,
        voxels: SystemParamItem<'w, '_, Self::Param>,
        pass: &mut TrackedRenderPass<'w>,
    ) -> RenderCommandResult {
        if let Ok(voxels) = voxels.get(item) {
            trace!("DrawVoxels: EntityRenderCommand");
            pass.set_vertex_buffer(0, voxels.points_buffer.slice(..));
            pass.set_vertex_buffer(1, voxels.colors_buffer.slice(..));
            pass.draw(0..(voxels.num_voxels as u32 * 2), 0..1);
        }
        RenderCommandResult::Success
    }
}
