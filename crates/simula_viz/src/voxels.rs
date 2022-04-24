use bevy::{
    ecs::system::{lifetimeless::SRes, SystemParamItem},
    pbr::MaterialPipeline,
    prelude::*,
    reflect::TypeUuid,
    render::{
        mesh::{Indices, Mesh, MeshVertexBufferLayout},
        render_asset::{PrepareAssetError, RenderAsset},
        render_resource::{Shader, *},
        renderer::RenderDevice,
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
pub struct VoxelsMesh {
    positions: Vec<[f32; 3]>,
    normals: Vec<[f32; 3]>,
    colors: Vec<u32>,
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
            ([sp.min_x, sp.min_y, sp.max_z], [0., 0., -1.0], sp.color.as_rgba_u32()),
            ([sp.max_x, sp.min_y, sp.max_z], [0., 0., -1.0], sp.color.as_rgba_u32()),
            ([sp.max_x, sp.max_y, sp.max_z], [0., 0., -1.0], sp.color.as_rgba_u32()),
            ([sp.min_x, sp.max_y, sp.max_z], [0., 0., -1.0], sp.color.as_rgba_u32()),
            // back
            ([sp.min_x, sp.max_y, sp.min_z], [0., 0., 1.0],  sp.color.as_rgba_u32()),
            ([sp.max_x, sp.max_y, sp.min_z], [0., 0., 1.0],  sp.color.as_rgba_u32()),
            ([sp.max_x, sp.min_y, sp.min_z], [0., 0., 1.0],  sp.color.as_rgba_u32()),
            ([sp.min_x, sp.min_y, sp.min_z], [0., 0., 1.0],  sp.color.as_rgba_u32()),
            // right
            ([sp.max_x, sp.min_y, sp.min_z], [1.0, 0., 0.],  sp.color.as_rgba_u32()),
            ([sp.max_x, sp.max_y, sp.min_z], [1.0, 0., 0.],  sp.color.as_rgba_u32()),
            ([sp.max_x, sp.max_y, sp.max_z], [1.0, 0., 0.],  sp.color.as_rgba_u32()),
            ([sp.max_x, sp.min_y, sp.max_z], [1.0, 0., 0.],  sp.color.as_rgba_u32()),
            // left
            ([sp.min_x, sp.min_y, sp.max_z], [-1.0, 0., 0.], sp.color.as_rgba_u32()),
            ([sp.min_x, sp.max_y, sp.max_z], [-1.0, 0., 0.], sp.color.as_rgba_u32()),
            ([sp.min_x, sp.max_y, sp.min_z], [-1.0, 0., 0.], sp.color.as_rgba_u32()),
            ([sp.min_x, sp.min_y, sp.min_z], [-1.0, 0., 0.], sp.color.as_rgba_u32()),
            // up
            ([sp.max_x, sp.max_y, sp.min_z], [0., 1.0, 0.],  sp.color.as_rgba_u32()),
            ([sp.min_x, sp.max_y, sp.min_z], [0., 1.0, 0.],  sp.color.as_rgba_u32()),
            ([sp.min_x, sp.max_y, sp.max_z], [0., 1.0, 0.],  sp.color.as_rgba_u32()),
            ([sp.max_x, sp.max_y, sp.max_z], [0., 1.0, 0.],  sp.color.as_rgba_u32()),
            // bottom
            ([sp.max_x, sp.min_y, sp.max_z], [0., -1.0, 0.], sp.color.as_rgba_u32()),
            ([sp.min_x, sp.min_y, sp.max_z], [0., -1.0, 0.], sp.color.as_rgba_u32()),
            ([sp.min_x, sp.min_y, sp.min_z], [0., -1.0, 0.], sp.color.as_rgba_u32()),
            ([sp.max_x, sp.min_y, sp.min_z], [0., -1.0, 0.], sp.color.as_rgba_u32()),
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

fn merge(voxels: &Vec<Voxel>) -> VoxelsMesh {
    let mut mesh = VoxelsMesh::default();
    voxels.into_iter().fold(&mut mesh, |mesh, voxel| {
        let voxel_mesh: VoxelsMesh = (*voxel).into();
        mesh.extend(&voxel_mesh);
        mesh
    });
    mesh
}

#[derive(Component, Clone)]
pub struct Voxels {
    pub voxels: Vec<Voxel>,
}

impl Default for Voxels {
    fn default() -> Self {
        Self { voxels: Vec::new() }
    }
}

#[derive(Bundle)]
pub struct VoxelsBundle {
    pub voxels: Voxels,
    pub mesh: Handle<Mesh>,
    pub material: Handle<VoxelsMaterial>,
    pub transform: Transform,
    pub global_transform: GlobalTransform,
    pub visibility: Visibility,
    pub computed_visibility: ComputedVisibility,
}

impl Default for VoxelsBundle {
    fn default() -> Self {
        VoxelsBundle {
            voxels: Voxels::default(),
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
#[uuid = "9c3b191e-c141-11ec-bf25-02a179e5df2c"]
pub struct VoxelsMaterial;

#[derive(Clone)]
pub struct GpuVoxelsMaterial {
    bind_group: BindGroup,
}

pub struct VoxelsPlugin;

impl Plugin for VoxelsPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugin(MaterialPlugin::<VoxelsMaterial>::default())
            .add_system(generate_voxels);
    }
}

fn generate_voxels(
    mut meshes: ResMut<Assets<Mesh>>,
    mut voxels: Query<
        (Entity, &mut Voxels, &ComputedVisibility, &Handle<Mesh>),
        With<Handle<VoxelsMaterial>>,
    >,
) {
    for (_entity, voxels, visibility, mesh_handle) in voxels.iter_mut() {
        if !visibility.is_visible {
            continue;
        }
        let voxel_mesh = merge(&voxels.voxels);
        if let Some(mesh) = meshes.get_mut(&mesh_handle.clone()) {
            voxel_mesh.with_mesh(mesh);
        }
    }
}

impl RenderAsset for VoxelsMaterial {
    type ExtractedAsset = VoxelsMaterial;
    type PreparedAsset = GpuVoxelsMaterial;
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

        Ok(GpuVoxelsMaterial { bind_group })
    }
}

impl Material for VoxelsMaterial {
    fn vertex_shader(asset_server: &AssetServer) -> Option<Handle<Shader>> {
        Some(asset_server.load("shaders/voxels.wgsl"))
    }

    fn fragment_shader(asset_server: &AssetServer) -> Option<Handle<Shader>> {
        Some(asset_server.load("shaders/voxels.wgsl"))
    }

    fn bind_group(render_asset: &<Self as RenderAsset>::PreparedAsset) -> &BindGroup {
        &render_asset.bind_group
    }

    fn bind_group_layout(render_device: &RenderDevice) -> BindGroupLayout {
        render_device.create_bind_group_layout(&BindGroupLayoutDescriptor {
            label: Some("voxels_bind_group_layout"),
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
            Mesh::ATTRIBUTE_NORMAL.at_shader_location(1),
            Mesh::ATTRIBUTE_UV_0.at_shader_location(2),
            Mesh::ATTRIBUTE_COLOR.at_shader_location(3),
        ])?;
        descriptor.vertex.buffers = vec![vertex_layout];
        Ok(())
    }
}
