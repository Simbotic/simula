use bevy::{
    core_pipeline::Opaque3d,
    ecs::system::{
        lifetimeless::{Read, SQuery, SRes},
        SystemParamItem,
    },
    pbr::MeshPipelineKey,
    prelude::*,
    render::{
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

pub const MAX_LINES: usize = 128000;
pub const MAX_POINTS: usize = MAX_LINES * 2;

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
    pub fn line(&mut self, start: Vec3, end: Vec3) {
        self.line_colored(start, end, Color::WHITE);
    }

    pub fn line_colored(&mut self, start: Vec3, end: Vec3, color: Color) {
        self.line_gradient(start, end, color, color);
    }

    pub fn line_gradient(&mut self, start: Vec3, end: Vec3, start_color: Color, end_color: Color) {
        let line = Line::new(start, end, start_color, end_color);
        if self.lines.len() == MAX_LINES {
            // bevy::log::warn!("Hit max lines, so replaced most recent line.");
            self.lines.pop();
        }
        self.lines.push(line);
    }
}

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
            label: Some("lines_time_uniform_buffer"),
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
    query: Query<(Entity, &Lines, &GlobalTransform), With<LinesMaterial>>,
) {
    let mut values = Vec::with_capacity(*previous_len);
    for (entity, lines, transform) in query.iter() {
        values.push((entity, (LinesMaterial, lines.clone(), transform.clone())));
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
        trace!("queue_lines: views.iter_mut()");
        let view_matrix = view.transform.compute_matrix();
        let view_row_2 = view_matrix.row(2);
        for (entity, transform) in material_lines.iter() {
            trace!("queue_lines: material_lines.iter()");

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
        label: Some("lines_time_bind_group"),
        layout: &pipeline.time_bind_group_layout,
        entries: &[BindGroupEntry {
            binding: 0,
            resource: time_meta.buffer.as_entire_binding(),
        }],
    });
    time_meta.bind_group = Some(bind_group);
}

#[derive(Component)]
struct ExtractedLines {
    num_lines: usize,
    points: Vec<[f32; 4]>,
    colors: Vec<[f32; 4]>,
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
        let num_lines = lines.lines.len();

        trace!("num_lines {}", num_lines);

        points.resize(num_lines * 2, [0f32; 4]);
        colors.resize(num_lines * 2, [0f32; 4]);

        for line in lines.lines.iter() {
            points[i] = line.start.extend(1.0).into();
            points[i + 1] = line.end.extend(1.0).into();
            colors[i] = line.color[0].as_rgba_f32().into();
            colors[i + 1] = line.color[1].as_rgba_f32().into();
            i += 2;
        }
        lines.lines = vec![];

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

    trace!("extract_lines {}", values.len());
    *previous_len = values.len();
    commands.insert_or_spawn_batch(values);
}

#[derive(Component)]
struct LinesMeta {
    num_lines: usize,
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
        let points: Vec<u8> = extracted_lines
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

        let colors: Vec<u8> = extracted_lines
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
            label: Some("lines_points_buffer"),
            size: points.len() as u64,
            usage: BufferUsages::VERTEX | BufferUsages::UNIFORM | BufferUsages::COPY_DST,
            mapped_at_creation: false,
        });

        let colors_buffer = render_device.create_buffer(&BufferDescriptor {
            label: Some("lines_colors_buffer"),
            size: colors.len() as u64,
            usage: BufferUsages::VERTEX | BufferUsages::UNIFORM | BufferUsages::COPY_DST,
            mapped_at_creation: false,
        });

        render_queue.write_buffer(&points_buffer, 0, bevy::core::cast_slice(&points));
        render_queue.write_buffer(&colors_buffer, 0, bevy::core::cast_slice(&colors));

        let bind_group = render_device.create_bind_group(&BindGroupDescriptor {
            label: Some("lines_create_bind_group"),
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
    line_bind_group_layout: BindGroupLayout,
    time_bind_group_layout: BindGroupLayout,
}

impl FromWorld for LinesPipeline {
    fn from_world(world: &mut World) -> Self {
        let world = world.cell();
        let asset_server = world.get_resource::<AssetServer>().unwrap();
        let shader = asset_server.load("shaders/lines.wgsl");

        let render_device = world.get_resource_mut::<RenderDevice>().unwrap();

        let view_bind_group_layout =
            render_device.create_bind_group_layout(&BindGroupLayoutDescriptor {
                label: Some("lines_view_bind_group_layout"),
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

        let line_bind_group_layout =
            render_device.create_bind_group_layout(&BindGroupLayoutDescriptor {
                label: Some("lines_bind_group_layout"),
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

        let time_bind_group_layout =
            render_device.create_bind_group_layout(&BindGroupLayoutDescriptor {
                label: Some("lines_time_bind_group_layout"),
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

        LinesPipeline {
            shader,
            view_bind_group_layout,
            line_bind_group_layout,
            time_bind_group_layout,
        }
    }
}

impl SpecializedPipeline for LinesPipeline {
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
                self.line_bind_group_layout.clone(),
                self.time_bind_group_layout.clone(),
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
            trace!("queue_view_bind_groups: views.iter()");
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
        if let Ok((view_uniform, lines_view_bind_group)) = view_query.get(view) {
            trace!("SetLinesViewBindGroup: EntityRenderCommand");
            pass.set_bind_group(I, &lines_view_bind_group.value, &[view_uniform.offset]);
        }

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
        if let Ok(lines_meta) = lines_meta.get(item) {
            trace!("SetLinesBindGroup: EntityRenderCommand");
            pass.set_bind_group(I, &lines_meta.bind_group, &[]);
        }

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
        if let Some(time_bind_group) = time_meta.into_inner().bind_group.as_ref() {
            trace!("SetLinesTimeBindGroup: EntityRenderCommand");
            pass.set_bind_group(I, time_bind_group, &[]);
        }

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
        if let Ok(lines) = lines.get(item) {
            trace!("DrawLines: EntityRenderCommand");
            pass.set_vertex_buffer(0, lines.points_buffer.slice(..));
            pass.set_vertex_buffer(1, lines.colors_buffer.slice(..));
            pass.draw(0..(lines.num_lines as u32 * 2), 0..1);
        }

        RenderCommandResult::Success
    }
}
