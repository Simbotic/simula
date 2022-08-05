use bevy::{
    pbr::{MaterialPipeline, MaterialPipelineKey},
    prelude::*,
    reflect::TypeUuid,
    render::{
        mesh::{Mesh, MeshVertexBufferLayout},
        render_resource::{
            AsBindGroup, RenderPipelineDescriptor, ShaderRef, SpecializedMeshPipelineError,
        },
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

#[derive(Default, AsBindGroup, TypeUuid, Debug, Clone)]
#[uuid = "6bb686a6-c2dc-11ec-89a7-02a179e5df2c"]
pub struct LinesMaterial {}

pub struct LinesPlugin;

impl Plugin for LinesPlugin {
    fn build(&self, app: &mut App) {
        app.add_system(generate_lines)
        .add_plugin(MaterialPlugin::<LinesMaterial>::default());
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
        if !visibility.is_visible() {
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
        colors.resize(num_lines * 2, [0f32; 4]);

        for (idx, line) in lines.lines.iter().enumerate() {
            let i = idx * 2;
            points[i] = line.start.into();
            points[i + 1] = line.end.into();
            colors[i] = line.start_color.as_rgba_f32().into();
            colors[i + 1] = line.end_color.as_rgba_f32().into();
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

impl Material for LinesMaterial {
    fn vertex_shader() -> ShaderRef {
        "shaders/lines.wgsl".into()
    }

    fn fragment_shader() -> ShaderRef {
        "shaders/lines.wgsl".into()
    }

    fn specialize(
        _pipeline: &MaterialPipeline<Self>,
        descriptor: &mut RenderPipelineDescriptor,
        layout: &MeshVertexBufferLayout,
        _key: MaterialPipelineKey<Self>,
    ) -> Result<(), SpecializedMeshPipelineError> {
        let vertex_layout = layout.get_layout(&[
            Mesh::ATTRIBUTE_POSITION.at_shader_location(0),
            Mesh::ATTRIBUTE_COLOR.at_shader_location(1),
        ])?;
        descriptor.vertex.buffers = vec![vertex_layout];
        Ok(())
    }
}