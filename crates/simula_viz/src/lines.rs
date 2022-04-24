use bevy::{
    ecs::system::{lifetimeless::SRes, SystemParamItem},
    pbr::MaterialPipeline,
    prelude::*,
    reflect::TypeUuid,
    render::{
        mesh::{Mesh, MeshVertexBufferLayout},
        render_asset::{PrepareAssetError, RenderAsset},
        render_resource::{Shader, *},
        renderer::RenderDevice,
    },
};

#[derive(Clone, Reflect, Default)]
pub struct Line {
    start: Vec3,
    end: Vec3,
    start_color: Color,
    end_color: Color,
}

impl Line {
    pub fn new(start: Vec3, end: Vec3, start_color: Color, end_color: Color) -> Self {
        Self {
            start,
            end,
            start_color,
            end_color,
        }
    }
}

pub const MAX_LINES: usize = 128000;
pub const MAX_POINTS: usize = MAX_LINES * 2;

#[derive(Component, Clone, Reflect)]
#[reflect(Component)]
pub struct Lines {
    #[reflect(ignore)]
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
            self.lines.pop();
        }
        self.lines.push(line);
    }
}

#[derive(Bundle)]
pub struct LinesBundle {
    pub lines: Lines,
    pub mesh: Handle<Mesh>,
    pub material: Handle<LinesMaterial>,
    pub transform: Transform,
    pub global_transform: GlobalTransform,
    pub visibility: Visibility,
    pub computed_visibility: ComputedVisibility,
}

impl Default for LinesBundle {
    fn default() -> Self {
        LinesBundle {
            lines: Lines::default(),
            mesh: Default::default(),
            material: Default::default(),
            transform: Transform::default(),
            global_transform: GlobalTransform::default(),
            visibility: Visibility::default(),
            computed_visibility: ComputedVisibility::default(),
        }
    }
}

#[derive(Debug, Clone, TypeUuid)]
#[uuid = "6bb686a6-c2dc-11ec-89a7-02a179e5df2c"]
pub struct LinesMaterial;

pub struct LinesPlugin;

impl Plugin for LinesPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugin(MaterialPlugin::<LinesMaterial>::default())
            .add_system(generate_lines);
    }
}

fn generate_lines(
    mut meshes: ResMut<Assets<Mesh>>,
    mut lines: Query<
        (Entity, &mut Lines, &ComputedVisibility, &Handle<Mesh>),
        With<Handle<LinesMaterial>>,
    >,
) {
    for (_entity, mut lines, visibility, mesh_handle) in lines.iter_mut() {
        if !visibility.is_visible {
            lines.lines.clear();
            continue;
        }

        let mut points = vec![];
        let mut normals = vec![];
        let mut uvs = vec![];
        let mut colors = vec![];

        let num_lines = lines.lines.len();

        points.resize(num_lines * 2, [0f32; 3]);
        normals.resize(num_lines * 2, [0f32; 3]);
        uvs.resize(num_lines * 2, [0f32; 2]);
        colors.resize(num_lines * 2, 0xFFFFFFFFu32);

        for (idx, line) in lines.lines.iter().enumerate() {
            let i = idx * 2;
            points[i] = line.start.into();
            points[i + 1] = line.end.into();
            colors[i] = line.start_color.as_rgba_u32();
            colors[i + 1] = line.end_color.as_rgba_u32();
        }

        if let Some(mesh) = meshes.get_mut(&mesh_handle.clone()) {
            mesh.insert_attribute(Mesh::ATTRIBUTE_POSITION, points);
            mesh.insert_attribute(Mesh::ATTRIBUTE_NORMAL, normals);
            mesh.insert_attribute(Mesh::ATTRIBUTE_UV_0, uvs);
            mesh.insert_attribute(Mesh::ATTRIBUTE_COLOR, colors);
        }

        lines.lines.clear();
    }
}

#[derive(Clone)]
pub struct GpuLinesMaterial {
    bind_group: BindGroup,
}

impl RenderAsset for LinesMaterial {
    type ExtractedAsset = LinesMaterial;
    type PreparedAsset = GpuLinesMaterial;
    type Param = (SRes<RenderDevice>, SRes<MaterialPipeline<Self>>);

    fn extract_asset(&self) -> Self::ExtractedAsset {
        self.clone()
    }

    fn prepare_asset(
        _extracted_asset: Self::ExtractedAsset,
        (render_device, material_pipeline): &mut SystemParamItem<Self::Param>,
    ) -> Result<Self::PreparedAsset, PrepareAssetError<Self::ExtractedAsset>> {
        let bind_group = render_device.create_bind_group(&BindGroupDescriptor {
            entries: &[],
            label: None,
            layout: &material_pipeline.material_layout,
        });

        Ok(GpuLinesMaterial { bind_group })
    }
}

impl Material for LinesMaterial {
    fn vertex_shader(asset_server: &AssetServer) -> Option<Handle<Shader>> {
        Some(asset_server.load("shaders/lines.wgsl"))
    }

    fn fragment_shader(asset_server: &AssetServer) -> Option<Handle<Shader>> {
        Some(asset_server.load("shaders/lines.wgsl"))
    }

    fn bind_group(render_asset: &<Self as RenderAsset>::PreparedAsset) -> &BindGroup {
        &render_asset.bind_group
    }

    fn bind_group_layout(render_device: &RenderDevice) -> BindGroupLayout {
        render_device.create_bind_group_layout(&BindGroupLayoutDescriptor {
            label: Some("lines_bind_group_layout"),
            entries: &[],
        })
    }

    fn specialize(
        _pipeline: &MaterialPipeline<Self>,
        descriptor: &mut RenderPipelineDescriptor,
        layout: &MeshVertexBufferLayout,
    ) -> Result<(), SpecializedMeshPipelineError> {
        let vertex_layout = layout.get_layout(&[
            Mesh::ATTRIBUTE_POSITION.at_shader_location(0),
            Mesh::ATTRIBUTE_COLOR.at_shader_location(1),
        ])?;
        descriptor.vertex.buffers = vec![vertex_layout];
        Ok(())
    }
}
