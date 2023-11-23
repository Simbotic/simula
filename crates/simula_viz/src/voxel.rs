use bevy::{
    pbr::{MaterialPipeline, MaterialPipelineKey},
    prelude::*,
    reflect::TypeUuid,
    render::{
        mesh::{Indices, Mesh, MeshVertexBufferLayout},
        render_resource::{
            AsBindGroup, PrimitiveTopology, RenderPipelineDescriptor, ShaderRef,
            SpecializedMeshPipelineError,
        },
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
        let raw_mesh: VoxelsMesh = voxel_box.into();
        raw_mesh.into()
    }
}

#[derive(Debug, Copy, Clone)]
pub struct Box {
    pub min: Vec3,
    pub max: Vec3,
    pub color: Color,
}

impl Box {
    pub fn new(x_length: f32, y_length: f32, z_length: f32, color: Color) -> Box {
        Box {
            min: Vec3::new(-x_length / 2.0, -y_length / 2.0, -z_length / 2.0),
            max: Vec3::new(x_length / 2.0, y_length / 2.0, z_length / 2.0),
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
        voxel_box.min += voxel.position;
        voxel_box.max += voxel.position;
        voxel_box
    }
}

#[derive(Default)]
pub struct VoxelsMesh {
    positions: Vec<[f32; 3]>,
    normals: Vec<[f32; 3]>,
    colors: Vec<[f32; 4]>,
    indices: Vec<u32>,
}

impl From<VoxelsMesh> for Mesh {
    fn from(voxel_mesh: VoxelsMesh) -> Self {
        let mut mesh = Mesh::new(PrimitiveTopology::TriangleList);
        let count = voxel_mesh.positions.len();
        mesh.insert_attribute(Mesh::ATTRIBUTE_POSITION, voxel_mesh.positions);
        mesh.insert_attribute(Mesh::ATTRIBUTE_NORMAL, voxel_mesh.normals);
        mesh.insert_attribute(Mesh::ATTRIBUTE_UV_0, vec![[0.0, 0.0]; count]);
        mesh.insert_attribute(Mesh::ATTRIBUTE_COLOR, voxel_mesh.colors);
        mesh.set_indices(Some(Indices::U32(voxel_mesh.indices)));
        mesh
    }
}

impl VoxelsMesh {
    pub fn with_mesh(&self, mesh: &mut Mesh) {
        let count = self.positions.len();
        mesh.insert_attribute(Mesh::ATTRIBUTE_POSITION, self.positions.clone());
        mesh.insert_attribute(Mesh::ATTRIBUTE_NORMAL, self.normals.clone());
        mesh.insert_attribute(Mesh::ATTRIBUTE_UV_0, vec![[0.0, 0.0]; count]);
        mesh.insert_attribute(Mesh::ATTRIBUTE_COLOR, self.colors.clone());
        mesh.set_indices(Some(Indices::U32(self.indices.clone())));
    }

    pub fn extend(&mut self, other: &VoxelsMesh) {
        let offset = self.positions.len() as u32;
        self.positions.extend(&other.positions);
        self.normals.extend(&other.normals);
        self.colors.extend(&other.colors);
        self.indices
            .extend(other.indices.iter().map(|i| i + offset));
    }
}

impl From<Box> for VoxelsMesh {
    fn from(sp: Box) -> Self {
        #[rustfmt::skip]
        let vertices = &[
            // front
            ([sp.min.x, sp.min.y, sp.max.z], [0., 0., -1.0], sp.color.as_rgba_f32()),
            ([sp.max.x, sp.min.y, sp.max.z], [0., 0., -1.0], sp.color.as_rgba_f32()),
            ([sp.max.x, sp.max.y, sp.max.z], [0., 0., -1.0], sp.color.as_rgba_f32()),
            ([sp.min.x, sp.max.y, sp.max.z], [0., 0., -1.0], sp.color.as_rgba_f32()),
            // back
            ([sp.min.x, sp.max.y, sp.min.z], [0., 0., 1.0],  sp.color.as_rgba_f32()),
            ([sp.max.x, sp.max.y, sp.min.z], [0., 0., 1.0],  sp.color.as_rgba_f32()),
            ([sp.max.x, sp.min.y, sp.min.z], [0., 0., 1.0],  sp.color.as_rgba_f32()),
            ([sp.min.x, sp.min.y, sp.min.z], [0., 0., 1.0],  sp.color.as_rgba_f32()),
            // right
            ([sp.max.x, sp.min.y, sp.min.z], [1.0, 0., 0.],  sp.color.as_rgba_f32()),
            ([sp.max.x, sp.max.y, sp.min.z], [1.0, 0., 0.],  sp.color.as_rgba_f32()),
            ([sp.max.x, sp.max.y, sp.max.z], [1.0, 0., 0.],  sp.color.as_rgba_f32()),
            ([sp.max.x, sp.min.y, sp.max.z], [1.0, 0., 0.],  sp.color.as_rgba_f32()),
            // left
            ([sp.min.x, sp.min.y, sp.max.z], [-1.0, 0., 0.], sp.color.as_rgba_f32()),
            ([sp.min.x, sp.max.y, sp.max.z], [-1.0, 0., 0.], sp.color.as_rgba_f32()),
            ([sp.min.x, sp.max.y, sp.min.z], [-1.0, 0., 0.], sp.color.as_rgba_f32()),
            ([sp.min.x, sp.min.y, sp.min.z], [-1.0, 0., 0.], sp.color.as_rgba_f32()),
            // up
            ([sp.max.x, sp.max.y, sp.min.z], [0., 1.0, 0.],  sp.color.as_rgba_f32()),
            ([sp.min.x, sp.max.y, sp.min.z], [0., 1.0, 0.],  sp.color.as_rgba_f32()),
            ([sp.min.x, sp.max.y, sp.max.z], [0., 1.0, 0.],  sp.color.as_rgba_f32()),
            ([sp.max.x, sp.max.y, sp.max.z], [0., 1.0, 0.],  sp.color.as_rgba_f32()),
            // bottom
            ([sp.max.x, sp.min.y, sp.max.z], [0., -1.0, 0.], sp.color.as_rgba_f32()),
            ([sp.min.x, sp.min.y, sp.max.z], [0., -1.0, 0.], sp.color.as_rgba_f32()),
            ([sp.min.x, sp.min.y, sp.min.z], [0., -1.0, 0.], sp.color.as_rgba_f32()),
            ([sp.max.x, sp.min.y, sp.min.z], [0., -1.0, 0.], sp.color.as_rgba_f32()),
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

        VoxelsMesh {
            positions,
            normals,
            colors,
            indices,
        }
    }
}

impl From<Voxel> for VoxelsMesh {
    fn from(voxel: Voxel) -> Self {
        let bx: Box = voxel.into();
        bx.into()
    }
}

fn merge(voxels: &[Voxel]) -> VoxelsMesh {
    let mut mesh = VoxelsMesh::default();
    voxels.iter().fold(&mut mesh, |mesh, voxel| {
        let voxel_mesh: VoxelsMesh = (*voxel).into();
        mesh.extend(&voxel_mesh);
        mesh
    });
    mesh
}

#[derive(Component, Clone, Default)]
pub struct Voxels {
    pub voxels: Vec<Voxel>,
}

#[derive(Bundle, Default)]
pub struct VoxelsBundle {
    pub voxels: Voxels,
    pub mesh: Handle<Mesh>,
    pub material: Handle<VoxelsMaterial>,
    pub transform: Transform,
    pub global_transform: GlobalTransform,
    pub visibility: Visibility,
    pub inherited_visibility: InheritedVisibility,
    pub view_visibility: ViewVisibility,
}

#[derive(Asset, TypePath, AsBindGroup, Debug, Clone, TypeUuid)]
#[uuid = "9c3b191e-c141-11ec-bf25-02a179e5df2c"]
pub struct VoxelsMaterial {}

pub struct VoxelsPlugin;

#[derive(Deref, Resource)]
pub struct VoxelMesh(Mesh);

impl Plugin for VoxelsPlugin {
    fn build(&self, app: &mut App) {
        // Add a voxel mesh that can be used as default by all voxels
        let mut mesh: Mesh = Mesh::new(PrimitiveTopology::TriangleList);
        mesh.insert_attribute(Mesh::ATTRIBUTE_POSITION, Vec::<[f32; 3]>::new());
        mesh.insert_attribute(Mesh::ATTRIBUTE_NORMAL, Vec::<[f32; 3]>::new());
        mesh.insert_attribute(Mesh::ATTRIBUTE_UV_0, Vec::<[f32; 2]>::new());
        mesh.insert_attribute(Mesh::ATTRIBUTE_COLOR, Vec::<[f32; 4]>::new());
        let voxel_mesh = VoxelMesh(mesh);
        app.insert_resource(voxel_mesh);

        app.add_plugins(MaterialPlugin::<VoxelsMaterial>::default())
            .add_systems(Update, generate_voxels);
    }
}

fn generate_voxels(
    mut meshes: ResMut<Assets<Mesh>>,
    mut voxels: Query<
        (
            Entity,
            &mut Voxels,
            &InheritedVisibility,
            &ViewVisibility,
            &Handle<Mesh>,
        ),
        With<Handle<VoxelsMaterial>>,
    >,
) {
    for (_entity, voxels, inherited_visibility, view_visibility, mesh_handle) in voxels.iter_mut() {
        if !inherited_visibility.get() && !view_visibility.get() {
            continue;
        }
        let voxel_mesh = merge(&voxels.voxels);
        if let Some(mesh) = meshes.get_mut(&mesh_handle.clone()) {
            voxel_mesh.with_mesh(mesh);
        }
    }
}

impl Material for VoxelsMaterial {
    fn vertex_shader() -> ShaderRef {
        "shaders/voxels.wgsl".into()
    }

    fn fragment_shader() -> ShaderRef {
        "shaders/voxels.wgsl".into()
    }

    fn specialize(
        _pipeline: &MaterialPipeline<Self>,
        descriptor: &mut RenderPipelineDescriptor,
        layout: &MeshVertexBufferLayout,
        _key: MaterialPipelineKey<Self>,
    ) -> Result<(), SpecializedMeshPipelineError> {
        let vertex_layout = layout.get_layout(&[
            Mesh::ATTRIBUTE_POSITION.at_shader_location(0),
            Mesh::ATTRIBUTE_NORMAL.at_shader_location(1),
            Mesh::ATTRIBUTE_UV_0.at_shader_location(2),
            Mesh::ATTRIBUTE_COLOR.at_shader_location(3),
        ])?;
        descriptor.vertex.buffers = vec![vertex_layout];
        Ok(())
    }
}
