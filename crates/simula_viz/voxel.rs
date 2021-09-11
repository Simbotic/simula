use bevy::{
    prelude::*,
    render::{
        mesh::{Indices, Mesh},
        pipeline::{PipelineDescriptor, PrimitiveTopology, RenderPipeline},
        shader::{ShaderStage, ShaderStages},
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

impl From<Voxel> for RawMesh {
    fn from(voxel: Voxel) -> Self {
        let bx: Box = voxel.into();
        bx.into()
    }
}

impl From<RawMesh> for Mesh {
    fn from(raw_mesh: RawMesh) -> Self {
        let mut mesh = Mesh::new(PrimitiveTopology::TriangleList);
        mesh.set_attribute(Mesh::ATTRIBUTE_POSITION, raw_mesh.positions);
        mesh.set_attribute(Mesh::ATTRIBUTE_NORMAL, raw_mesh.normals);
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

pub fn setup(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut pipelines: ResMut<Assets<PipelineDescriptor>>,
    mut shaders: ResMut<Assets<Shader>>,
) {
    let pipeline_handle = pipelines.add(PipelineDescriptor::default_config(ShaderStages {
        vertex: shaders.add(Shader::from_glsl(ShaderStage::Vertex, VERTEX_SHADER)),
        fragment: Some(shaders.add(Shader::from_glsl(ShaderStage::Fragment, FRAGMENT_SHADER))),
    }));

    let voxel = Voxel {
        position: Vec3::new(0., 0., 0.),
        size: 1.,
        color: Color::CYAN,
    };

    commands
        .spawn_bundle(MeshBundle {
            mesh: meshes.add(voxel.into()),
            render_pipelines: RenderPipelines::from_pipelines(vec![RenderPipeline::new(
                pipeline_handle,
            )]),
            ..Default::default()
        })
        .with_children(|parent| {
            parent.spawn_bundle(super::axes::AxesBundle {
                axes: super::axes::Axes {
                    size: 10.,
                    ..Default::default()
                },
                ..Default::default()
            });
        });
}

pub const VERTEX_SHADER: &str = include_str!("../../assets/shaders/voxel.vert");

pub const FRAGMENT_SHADER: &str = include_str!("../../assets/shaders/voxel.frag");
