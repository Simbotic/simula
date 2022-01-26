use bevy::{
    core_pipeline::Opaque3d,
    ecs::system::{
        lifetimeless::{Read, SQuery, SRes},
        SystemParamItem,
    },
    pbr::{
        DrawMesh, MeshPipeline, MeshPipelineKey, MeshUniform, SetMeshBindGroup,
        SetMeshViewBindGroup,
    },
    prelude::*,
    render::{
        mesh::{GpuBufferInfo, VertexAttributeValues},
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
        view::{
            ComputedVisibility, ExtractedView, Msaa, ViewUniform, ViewUniformOffset, ViewUniforms,
            Visibility,
        },
        RenderApp, RenderStage,
    },
};

#[derive(Bundle, Default)]
pub struct LinesBundle {
    pub lines: Lines,
    pub material: LinesMaterial,
    pub transform: Transform,
    pub global_transform: GlobalTransform,
}

#[derive(Component, Default)]
pub struct LinesMaterial;

pub struct LinesPlugin;

impl Plugin for LinesPlugin {
    fn build(&self, app: &mut App) {
        let render_device = app.world.get_resource::<RenderDevice>().unwrap();

        let time_buffer = render_device.create_buffer(&BufferDescriptor {
            label: Some("time uniform buffer"),
            size: std::mem::size_of::<f32>() as u64,
            usage: BufferUsages::UNIFORM | BufferUsages::COPY_DST,
            mapped_at_creation: false,
        });

        app.sub_app_mut(RenderApp)
            .insert_resource(TimeMeta {
                buffer: time_buffer,
                bind_group: None,
            })
            .add_render_command::<Opaque3d, DrawLinesCustom>()
            .init_resource::<LinesPipeline>()
            .init_resource::<SpecializedPipelines<LinesPipeline>>()
            .add_system_to_stage(RenderStage::Extract, extract_time)
            .add_system_to_stage(RenderStage::Extract, extract_lines_material)
            .add_system_to_stage(RenderStage::Extract, extract_lines)
            .add_system_to_stage(RenderStage::Prepare, prepare_time)
            .add_system_to_stage(RenderStage::Prepare, prepare_lines)
            .add_system_to_stage(RenderStage::Queue, queue_lines)
            .add_system_to_stage(RenderStage::Queue, queue_time_bind_group)
            .add_system_to_stage(RenderStage::Queue, queue_view_bind_groups);
    }
}

// extract the `LinesMaterial` component into the render world
fn extract_lines_material(
    mut previous_len: Local<usize>,
    mut commands: Commands,
    query: Query<(Entity, &Lines), With<LinesMaterial>>,
) {
    let mut values = Vec::with_capacity(*previous_len);
    for (entity, lines) in query.iter() {
        values.push((entity, (LinesMaterial, lines.clone())));
    }
    *previous_len = values.len();
    commands.insert_or_spawn_batch(values);
}

// add each entity with a mesh and a `LinesMaterial` to every view's `Opaque3d` render phase using the `LinesPipeline`
fn queue_lines(
    opaque_3d_draw_functions: Res<DrawFunctions<Opaque3d>>,
    lines_pipeline: Res<LinesPipeline>,
    msaa: Res<Msaa>,
    mut pipelines: ResMut<SpecializedPipelines<LinesPipeline>>,
    mut pipeline_cache: ResMut<RenderPipelineCache>,
    material_lines: Query<(Entity, &GlobalTransform), With<LinesMaterial>>,
    mut views: Query<(&ExtractedView, &mut RenderPhase<Opaque3d>)>,
) {
    let draw_lines = opaque_3d_draw_functions
        .read()
        .get_id::<DrawLinesCustom>()
        .unwrap();

    let key = MeshPipelineKey::from_msaa_samples(msaa.samples)
        | MeshPipelineKey::from_primitive_topology(PrimitiveTopology::LineList);
    let pipeline = pipelines.specialize(&mut pipeline_cache, &lines_pipeline, key);

    for (view, mut opaque_phase) in views.iter_mut() {
        println!("queue_lines: views.iter_mut()");
        let view_matrix = view.transform.compute_matrix();
        let view_row_2 = view_matrix.row(2);
        for (entity, transform) in material_lines.iter() {
            println!("queue_lines: material_lines.iter()");

            opaque_phase.add(Opaque3d {
                entity,
                pipeline,
                draw_function: draw_lines,
                distance: view_row_2.dot(transform.compute_matrix().col(3)),
            });
        }
    }
}

#[derive(Default)]
struct ExtractedTime {
    seconds_since_startup: f32,
}

// extract the passed time into a resource in the render world
fn extract_time(mut commands: Commands, time: Res<Time>) {
    commands.insert_resource(ExtractedTime {
        seconds_since_startup: time.seconds_since_startup() as f32,
    });
}

struct TimeMeta {
    buffer: Buffer,
    bind_group: Option<BindGroup>,
}

// write the extracted time into the corresponding uniform buffer
fn prepare_time(
    time: Res<ExtractedTime>,
    time_meta: ResMut<TimeMeta>,
    render_queue: Res<RenderQueue>,
) {
    render_queue.write_buffer(
        &time_meta.buffer,
        0,
        bevy::core::cast_slice(&[time.seconds_since_startup]),
    );
}

// create a bind group for the time uniform buffer
fn queue_time_bind_group(
    render_device: Res<RenderDevice>,
    mut time_meta: ResMut<TimeMeta>,
    pipeline: Res<LinesPipeline>,
) {
    let bind_group = render_device.create_bind_group(&BindGroupDescriptor {
        label: None,
        layout: &pipeline.time_bind_group_layout,
        entries: &[BindGroupEntry {
            binding: 0,
            resource: time_meta.buffer.as_entire_binding(),
        }],
    });
    time_meta.bind_group = Some(bind_group);
}

#[derive(Component, Default)]
struct ExtractedLines {
    num_lines: u32,
    points: Vec<Vec4>,
    colors: Vec<Vec4>,
}

// extract the lines into a resource in the render world
fn extract_lines(
    mut previous_len: Local<usize>,
    mut commands: Commands,
    mut lines: Query<(Entity, &mut Lines), With<LinesMaterial>>,
) {
    let mut values = Vec::with_capacity(*previous_len);
    for (entity, mut lines) in lines.iter_mut() {
        let mut points = vec![];
        let mut colors = vec![];

        let mut i = 0;
        let count = lines.lines.len();

        points.resize(count * 2, Vec4::ZERO);
        colors.resize(count * 2, Vec4::ZERO);

        for line in lines.lines.iter() {
            points[i] = line.start.extend(0.0);
            points[i + 1] = line.end.extend(0.0);
            colors[i] = line.color[0].as_rgba_f32().into();
            colors[i + 1] = line.color[1].as_rgba_f32().into();
            i += 2;
        }

        lines.lines = vec![];

        let size = if count > MAX_LINES {
            bevy::log::warn!(
                "Lines: Maximum number of lines exceeded: line count: {}, max lines: {}",
                count,
                MAX_LINES
            );
            MAX_LINES
        } else {
            count
        };
        let num_lines = size as u32;

        println!("num_lines {}", num_lines);

        values.push((
            entity,
            (
                LinesMaterial,
                ExtractedLines {
                    num_lines,
                    points,
                    colors,
                },
            ),
        ));
    }

    println!("values {}", values.len());

    *previous_len = values.len();
    commands.insert_or_spawn_batch(values);
}

#[derive(Component)]
struct LinesMeta {
    num_lines: u32,
    points_buffer: Buffer,
    colors_buffer: Buffer,
    bind_group: BindGroup,
}

fn prepare_lines(
    render_device: Res<RenderDevice>,
    render_queue: Res<RenderQueue>,
    pipeline: Res<LinesPipeline>,
    mut commands: Commands,
    lines: Query<(Entity, &mut ExtractedLines), With<LinesMaterial>>,
) {
    for (entity, extracted_lines) in lines.iter() {
        let points = extracted_lines
            .points
            .iter()
            .flat_map(|point| {
                point
                    .as_std140()
                    .as_bytes()
                    .iter()
                    .cloned()
                    .collect::<Vec<u8>>()
            })
            .collect::<Vec<u8>>();

        let colors = extracted_lines
            .colors
            .iter()
            .flat_map(|color| {
                color
                    .as_std140()
                    .as_bytes()
                    .iter()
                    .cloned()
                    .collect::<Vec<u8>>()
            })
            .collect::<Vec<u8>>();

        // println!("points.len() {}", points.len());

        // if points.len() == 0 {
        //     continue;
        // }

        println!("points.len() {}", points.len());

        let points_buffer = render_device.create_buffer(&BufferDescriptor {
            label: Some("lines points buffer"),
            size: points.len() as u64,
            usage: BufferUsages::UNIFORM | BufferUsages::COPY_DST,
            mapped_at_creation: false,
        });

        let colors_buffer = render_device.create_buffer(&BufferDescriptor {
            label: Some("lines colors buffer"),
            size: colors.len() as u64,
            usage: BufferUsages::UNIFORM | BufferUsages::COPY_DST,
            mapped_at_creation: false,
        });

        render_queue.write_buffer(&points_buffer, 0, bevy::core::cast_slice(&points));
        render_queue.write_buffer(&colors_buffer, 0, bevy::core::cast_slice(&colors));

        let bind_group = render_device.create_bind_group(&BindGroupDescriptor {
            label: Some("lines create bind group"),
            layout: &pipeline.line_bind_group_layout,
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

        let lines_meta = LinesMeta {
            num_lines: extracted_lines.num_lines,
            points_buffer,
            colors_buffer,
            bind_group,
        };

        commands.entity(entity).insert(lines_meta);
    }
}

pub struct LinesPipeline {
    shader: Handle<Shader>,
    view_bind_group_layout: BindGroupLayout,
    time_bind_group_layout: BindGroupLayout,
    line_bind_group_layout: BindGroupLayout,
}

impl FromWorld for LinesPipeline {
    fn from_world(world: &mut World) -> Self {
        let world = world.cell();
        let asset_server = world.get_resource::<AssetServer>().unwrap();
        let shader = asset_server.load("shaders/lines.wgsl");

        let render_device = world.get_resource_mut::<RenderDevice>().unwrap();

        let view_bind_group_layout =
            render_device.create_bind_group_layout(&BindGroupLayoutDescriptor {
                label: Some("view bind group"),
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

        let time_bind_group_layout =
            render_device.create_bind_group_layout(&BindGroupLayoutDescriptor {
                label: Some("time bind group"),
                entries: &[BindGroupLayoutEntry {
                    binding: 0,
                    visibility: ShaderStages::FRAGMENT,
                    ty: BindingType::Buffer {
                        ty: BufferBindingType::Uniform,
                        has_dynamic_offset: false,
                        min_binding_size: BufferSize::new(std::mem::size_of::<f32>() as u64),
                    },
                    count: None,
                }],
            });

        let line_bind_group_layout =
            render_device.create_bind_group_layout(&BindGroupLayoutDescriptor {
                label: Some("lines bind group"),
                entries: &[
                    BindGroupLayoutEntry {
                        binding: 0,
                        visibility: ShaderStages::VERTEX,
                        ty: BindingType::Buffer {
                            ty: BufferBindingType::Uniform,
                            has_dynamic_offset: true,
                            min_binding_size: BufferSize::new(0),
                        },
                        count: None,
                    },
                    BindGroupLayoutEntry {
                        binding: 1,
                        visibility: ShaderStages::VERTEX,
                        ty: BindingType::Buffer {
                            ty: BufferBindingType::Uniform,
                            has_dynamic_offset: true,
                            min_binding_size: BufferSize::new(0),
                        },
                        count: None,
                    },
                ],
            });

        LinesPipeline {
            shader,
            view_bind_group_layout,
            time_bind_group_layout,
            line_bind_group_layout,
        }
    }
}

impl SpecializedPipeline for LinesPipeline {
    type Key = MeshPipelineKey;

    fn specialize(&self, key: Self::Key) -> RenderPipelineDescriptor {
        let shader_defs = Vec::new();

        let vertex_array_stride = 32;

        let vertex_attributes = vec![
            // Points
            VertexAttribute {
                format: VertexFormat::Float32x4,
                offset: 0,
                shader_location: 0,
            },
            // Colors
            VertexAttribute {
                format: VertexFormat::Float32x4,
                offset: 16,
                shader_location: 1,
            },
        ];

        RenderPipelineDescriptor {
            vertex: VertexState {
                shader: self.shader.clone(),
                entry_point: "vertex".into(),
                shader_defs: shader_defs.clone(),
                buffers: vec![VertexBufferLayout {
                    array_stride: vertex_array_stride,
                    step_mode: VertexStepMode::Vertex,
                    attributes: vertex_attributes,
                }],
            },
            fragment: Some(FragmentState {
                shader: self.shader.clone(),
                shader_defs,
                entry_point: "fragment".into(),
                targets: vec![ColorTargetState {
                    format: TextureFormat::bevy_default(),
                    blend: Some(BlendState::ALPHA_BLENDING),
                    write_mask: ColorWrites::ALL,
                }],
            }),
            layout: Some(vec![
                self.view_bind_group_layout.clone(),
                self.time_bind_group_layout.clone(),
                self.line_bind_group_layout.clone(),
            ]),
            primitive: PrimitiveState {
                front_face: FrontFace::Ccw,
                cull_mode: Some(Face::Back),
                unclipped_depth: false,
                polygon_mode: PolygonMode::Line,
                conservative: false,
                topology: key.primitive_topology(),
                strip_index_format: None,
            },
            depth_stencil: None,
            multisample: MultisampleState {
                count: key.msaa_samples(),
                mask: !0,
                alpha_to_coverage_enabled: false,
            },
            label: Some("lines_pipeline".into()),
        }
    }
}

type DrawLinesCustom = (
    SetItemPipeline,
    SetLinesViewBindGroup<0>,
    SetLinesBindGroup<1>,
    SetTimeBindGroup<2>,
    DrawLines,
);

#[derive(Component)]
pub struct LinesViewBindGroup {
    pub value: BindGroup,
}

pub fn queue_view_bind_groups(
    mut commands: Commands,
    render_device: Res<RenderDevice>,
    lines_pipeline: Res<LinesPipeline>,
    view_uniforms: Res<ViewUniforms>,
    views: Query<Entity>,
) {
    if let Some(view_binding) = view_uniforms.uniforms.binding() {
        for entity in views.iter() {
            println!("queue_view_bind_groups: views.iter()");
            let view_bind_group = render_device.create_bind_group(&BindGroupDescriptor {
                entries: &[BindGroupEntry {
                    binding: 0,
                    resource: view_binding.clone(),
                }],
                label: Some("lines_view_bind_group"),
                layout: &lines_pipeline.view_bind_group_layout,
            });

            commands.entity(entity).insert(LinesViewBindGroup {
                value: view_bind_group,
            });
        }
    }
}

pub struct SetLinesViewBindGroup<const I: usize>;
impl<const I: usize> EntityRenderCommand for SetLinesViewBindGroup<I> {
    type Param = SQuery<(Read<ViewUniformOffset>, Read<LinesViewBindGroup>)>;

    #[inline]
    fn render<'w>(
        view: Entity,
        _item: Entity,
        view_query: SystemParamItem<'w, '_, Self::Param>,
        pass: &mut TrackedRenderPass<'w>,
    ) -> RenderCommandResult {
        println!("SetLinesViewBindGroup: EntityRenderCommand");
        let (view_uniform, lines_view_bind_group) = view_query.get(view).unwrap();
        pass.set_bind_group(I, &lines_view_bind_group.value, &[view_uniform.offset]);

        RenderCommandResult::Success
    }
}

struct SetTimeBindGroup<const I: usize>;
impl<const I: usize> EntityRenderCommand for SetTimeBindGroup<I> {
    type Param = SRes<TimeMeta>;

    fn render<'w>(
        _view: Entity,
        _item: Entity,
        time_meta: SystemParamItem<'w, '_, Self::Param>,
        pass: &mut TrackedRenderPass<'w>,
    ) -> RenderCommandResult {
        println!("SetTimeBindGroup: EntityRenderCommand");
        let time_bind_group = time_meta.into_inner().bind_group.as_ref().unwrap();
        pass.set_bind_group(I, time_bind_group, &[]);

        RenderCommandResult::Success
    }
}

struct SetLinesBindGroup<const I: usize>;
impl<const I: usize> EntityRenderCommand for SetLinesBindGroup<I> {
    type Param = SQuery<Read<LinesMeta>>;

    fn render<'w>(
        _view: Entity,
        item: Entity,
        lines_meta: SystemParamItem<'w, '_, Self::Param>,
        pass: &mut TrackedRenderPass<'w>,
    ) -> RenderCommandResult {
        println!("SetLinesBindGroup: EntityRenderCommand");
        let lines_meta = lines_meta.get(item).unwrap();
        pass.set_bind_group(I, &lines_meta.bind_group, &[]);

        RenderCommandResult::Success
    }
}

struct DrawLines;
impl EntityRenderCommand for DrawLines {
    type Param = SQuery<Read<LinesMeta>>;

    #[inline]
    fn render<'w>(
        _view: Entity,
        item: Entity,
        lines: SystemParamItem<'w, '_, Self::Param>,
        pass: &mut TrackedRenderPass<'w>,
    ) -> RenderCommandResult {
        println!("DrawLines: EntityRenderCommand");
        let lines = lines.get(item).unwrap();

        pass.draw(0..(lines.num_lines * 2), 0..1);

        println!("pass.draw");

        RenderCommandResult::Success
        // let mesh_handle = mesh_query.get(item).unwrap();
        // if let Some(gpu_mesh) = meshes.into_inner().get(mesh_handle) {
        //     pass.set_vertex_buffer(0, gpu_mesh.vertex_buffer.slice(..));
        //     match &gpu_mesh.buffer_info {
        //         GpuBufferInfo::Indexed {
        //             buffer,
        //             index_format,
        //             count,
        //         } => {
        //             panic!("GpuBufferInfo::Indexed");
        //             // pass.set_index_buffer(buffer.slice(..), 0, *index_format);
        //             // pass.draw_indexed(0..*count, 0, 0..1);
        //         }
        //         GpuBufferInfo::NonIndexed { vertex_count } => {
        //             pass.draw(0..*vertex_count, 0..1);
        //         }
        //     }
        //     RenderCommandResult::Success
        // } else {
        //     RenderCommandResult::Failure
        // }
    }
}

/// A single line, usually initialized by helper methods on `Lines` instead of directly.
#[derive(Clone)]
pub struct Line {
    start: Vec3,
    end: Vec3,
    color: [Color; 2],
}

impl Line {
    pub fn new(start: Vec3, end: Vec3, start_color: Color, end_color: Color) -> Self {
        Self {
            start,
            end,
            color: [start_color, end_color],
        }
    }
}

/// Maximum number of unique lines to draw at once.
pub const MAX_LINES: usize = 128000;
/// Maximum number of points.
pub const MAX_POINTS: usize = MAX_LINES * 2;

fn create_mesh() -> Mesh {
    let mut mesh = Mesh::new(PrimitiveTopology::LineList);
    let positions = vec![[0.0, 0.0, 0.0]; MAX_LINES * 2];
    mesh.set_attribute(
        Mesh::ATTRIBUTE_POSITION,
        VertexAttributeValues::Float32x3(positions.into()),
    );
    mesh
}

/// Bevy resource providing facilities to draw lines.
///
/// # Usage
/// ```
/// // Draws 3 horizontal lines, which disappear after 1 frame.
/// fn some_system(mut lines: ResMut<Lines>) {
///     lines.line(Vec3::new(-1.0, 1.0, 0.0), Vec3::new(1.0, 1.0, 0.0), 0.0);
///     lines.line_colored(
///         Vec3::new(-1.0, 0.0, 0.0),
///         Vec3::new(1.0, 0.0, 0.0),
///         0.0,
///         Color::WHITE
///     );
///     lines.line_gradient(
///         Vec3::new(-1.0, -1.0, 0.0),
///         Vec3::new(1.0, -1.0, 0.0),
///         0.0,
///         Color::WHITE, Color::PINK
///     );
/// }
/// ```
///
/// # Properties
///
/// * `lines` - A `Vec` of `Line`s that is **cleared by the system every frame**.
/// Normally you don't want to touch this, and it may go private in future releases.
/// * `user_lines` - A Vec of `Line`s that is **not cleared by the system every frame**.
/// Use this for inserting persistent lines and generally having control over how lines are collected.
/// * `depth_test` - Enable/disable depth testing, i.e. whether lines should be drawn behind other
/// geometry.
#[derive(Component, Clone)]
pub struct Lines {
    pub lines: Vec<Line>,
}

impl Default for Lines {
    fn default() -> Self {
        Self { lines: Vec::new() }
    }
}

impl Lines {
    /// Draw a line in world space, or update an existing line
    ///
    /// # Arguments
    ///
    /// * `start` - The start of the line in world space
    /// * `end` - The end of the line in world space
    pub fn line(&mut self, start: Vec3, end: Vec3) {
        self.line_colored(start, end, Color::WHITE);
    }

    /// Draw a line in world space with a specified color, or update an existing line
    ///
    /// # Arguments
    ///
    /// * `start` - The start of the line in world space
    /// * `end` - The end of the line in world space
    /// * `color` - Line color
    pub fn line_colored(&mut self, start: Vec3, end: Vec3, color: Color) {
        self.line_gradient(start, end, color, color);
    }

    /// Draw a line in world space with a specified gradient color, or update an existing line
    ///
    /// # Arguments
    ///
    /// * `start` - The start of the line in world space
    /// * `end` - The end of the line in world space
    /// * `start_color` - Line color
    /// * `end_color` - Line color
    pub fn line_gradient(&mut self, start: Vec3, end: Vec3, start_color: Color, end_color: Color) {
        let line = Line::new(start, end, start_color, end_color);

        // If we are at maximum capacity, we push the first line out.
        if self.lines.len() == MAX_LINES {
            //bevy::log::warn!("Hit max lines, so replaced most recent line.");
            self.lines.pop();
        }

        self.lines.push(line);
    }
}

fn draw_lines(// mut assets: ResMut<Assets<LineMaterial>>,
    // mut lines: ResMut<Lines>,
    // time: Res<Time>,
    // query: Query<&Handle<LineMaterial>>,
) {
    // One line changing makes us update all lines.
    // We can probably resolve this is it becomes a problem -- consider creating a number of "Line" entities to
    // split up the processing.
    // This has been removed due to needing to redraw every frame now, but the logic is reasonable and
    // may be re-added at some point.
    //if !lines.dirty {
    //return;
    //}
    // for line_handle in query.iter() {
    //     // This could probably be faster if we can simplify to a memcpy instead.
    //     if let Some(shader) = assets.get_mut(line_handle) {
    //         let mut i = 0;
    //         let all_lines = lines.lines.iter().chain(lines.user_lines.iter());
    //         for line in all_lines {
    //             shader.points[i] = line.start.extend(0.0);
    //             shader.points[i + 1] = line.end.extend(0.0);
    //             shader.colors[i] = line.color[0].as_rgba_f32().into();
    //             shader.colors[i + 1] = line.color[1].as_rgba_f32().into();

    //             i += 2;
    //         }

    //         let count = lines.lines.len() + lines.user_lines.len();
    //         let size = if count > MAX_LINES {
    //             bevy::log::warn!(
    //                 "Lines: Maximum number of lines exceeded: line count: {}, max lines: {}",
    //                 count,
    //                 MAX_LINES
    //             );
    //             MAX_LINES
    //         } else {
    //             count
    //         };

    //         shader.num_lines = size as u32; // Minimum size to send to shader is 4 bytes.
    //     }
    // }

    // let mut i = 0;
    // let mut len = lines.lines.len();
    // while i != len {
    //     lines.lines[i].duration -= time.delta_seconds();
    //     if lines.lines[i].duration < 0.0 {
    //         lines.lines.swap(i, len - 1);
    //         len -= 1;
    //     } else {
    //         i += 1;
    //     }
    // }

    // lines.lines.truncate(len);
}
