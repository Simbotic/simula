use bevy::{
    core_pipeline::core_3d::Transparent3d,
    ecs::system::{lifetimeless::*, SystemParamItem},
    math::prelude::*,
    pbr::{
        MeshPipeline, MeshPipelineKey, RenderMeshInstances, SetMeshBindGroup, SetMeshViewBindGroup,
    },
    prelude::*,
    render::{
        extract_component::{ExtractComponent, ExtractComponentPlugin},
        mesh::{GpuBufferInfo, MeshVertexBufferLayout},
        render_asset::RenderAssets,
        render_phase::{
            AddRenderCommand, DrawFunctions, PhaseItem, RenderCommand, RenderCommandResult,
            RenderPhase, SetItemPipeline, TrackedRenderPass,
        },
        render_resource::*,
        renderer::RenderDevice,
        view::{ExtractedView, Msaa},
        RenderApp, RenderSet,
    },
};
use bytemuck::{Pod, Zeroable};

#[derive(Component, Default, Debug, Deref)]
pub struct Pointcloud(pub Vec<PointData>);
impl ExtractComponent for Pointcloud {
    type Query = &'static Pointcloud;
    type Filter = ();
    type Out = Self;

    fn extract_component(item: bevy::ecs::query::QueryItem<Self::Query>) -> Option<Self::Out> {
        Some(Pointcloud(item.0.clone()))
    }
}

pub struct PointcloudPlugin;

impl Plugin for PointcloudPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins(ExtractComponentPlugin::<Pointcloud>::default());
        app.sub_app_mut(RenderApp)
            .add_render_command::<Transparent3d, DrawCustom>()
            .init_resource::<PointcloudPipeline>()
            .init_resource::<SpecializedMeshPipelines<PointcloudPipeline>>()
            // .add_systems(Update, queue_pointclouds.in_set(RenderSet::Queue))
            .add_systems(Update, prepare_pointclouds.in_set(RenderSet::Prepare));
    }
}

#[derive(Clone, Copy, Pod, Zeroable, Default, Debug)]
#[repr(C)]
pub struct PointData {
    pub position: Vec3,
    pub scale: f32,
    pub color: [f32; 4],
}

// #[allow(clippy::too_many_arguments)]
// fn queue_pointclouds(
//     transparent_3d_draw_functions: Res<DrawFunctions<Transparent3d>>,
//     custom_pipeline: Res<PointcloudPipeline>,
//     msaa: Res<Msaa>,
//     mut pipelines: ResMut<SpecializedMeshPipelines<PointcloudPipeline>>,
//     mut pipeline_cache: ResMut<PipelineCache>,
//     meshes: Res<RenderAssets<Mesh>>,
//     material_meshes: Query<(Entity, &Handle<Mesh>), (With<Handle<Mesh>>, With<Pointcloud>)>,
//     mut views: Query<(&ExtractedView, &mut RenderPhase<Transparent3d>)>,
//     render_mesh_instances: Res<RenderMeshInstances>,
// ) {
//     let draw_custom = transparent_3d_draw_functions
//         .read()
//         .get_id::<DrawCustom>()
//         .unwrap();

//     let msaa_key = MeshPipelineKey::from_msaa_samples(msaa.samples());

//     for (view, mut transparent_phase) in views.iter_mut() {
//         let view_matrix = view.transform.compute_matrix();
//         let view_row_2 = view_matrix.row(2);
//         for (entity, mesh_handle) in material_meshes.iter() {
//             if let Some(mesh) = meshes.get(mesh_handle) {
//                 let Some(mesh_instance) = render_mesh_instances.get(&entity) else {
//                     continue;
//                 };
//                 let key =
//                     msaa_key | MeshPipelineKey::from_primitive_topology(mesh.primitive_topology);
//                 let pipeline = pipelines
//                     .specialize(&mut pipeline_cache, &custom_pipeline, key, &mesh.layout)
//                     .unwrap();
//                 // TODO: Check with Alex
//                 transparent_phase.add(Transparent3d {
//                     entity,
//                     pipeline,
//                     draw_function: draw_custom,
//                     distance: view_row_2.dot(mesh_instance.transforms.transform),
//                     batch_range: 0..1,
//                     dynamic_offset: None,
//                 });
//             }
//         }
//     }
// }

#[derive(Component)]
struct InstanceBuffer {
    buffer: Buffer,
    length: usize,
}

fn prepare_pointclouds(
    mut commands: Commands,
    query: Query<(Entity, &Pointcloud)>,
    render_device: Res<RenderDevice>,
) {
    for (entity, instance_data) in query.iter() {
        let buffer = render_device.create_buffer_with_data(&BufferInitDescriptor {
            label: Some("pointcloud data buffer"),
            contents: bytemuck::cast_slice(instance_data.as_slice()),
            usage: BufferUsages::VERTEX | BufferUsages::COPY_DST,
        });
        commands.entity(entity).insert(InstanceBuffer {
            buffer,
            length: instance_data.len(),
        });
    }
}

#[derive(Resource)]
struct PointcloudPipeline {
    shader: Handle<Shader>,
    mesh_pipeline: MeshPipeline,
}

impl FromWorld for PointcloudPipeline {
    fn from_world(world: &mut World) -> Self {
        let world = world.cell();
        let asset_server = world.get_resource::<AssetServer>().unwrap();
        let shader = asset_server.load("shaders/pointcloud.wgsl");

        let mesh_pipeline = world.get_resource::<MeshPipeline>().unwrap();

        PointcloudPipeline {
            shader,
            mesh_pipeline: mesh_pipeline.clone(),
        }
    }
}

impl SpecializedMeshPipeline for PointcloudPipeline {
    type Key = MeshPipelineKey;

    fn specialize(
        &self,
        key: Self::Key,
        layout: &MeshVertexBufferLayout,
    ) -> Result<RenderPipelineDescriptor, SpecializedMeshPipelineError> {
        let mut descriptor = self.mesh_pipeline.specialize(key, layout)?;
        descriptor.vertex.shader = self.shader.clone();
        descriptor.vertex.buffers.push(VertexBufferLayout {
            array_stride: std::mem::size_of::<PointData>() as u64,
            step_mode: VertexStepMode::Instance,
            attributes: vec![
                VertexAttribute {
                    format: VertexFormat::Float32x4,
                    offset: 0,
                    shader_location: 3, // shader locations 0-2 are taken up by Position, Normal and UV attributes
                },
                VertexAttribute {
                    format: VertexFormat::Float32x4,
                    offset: VertexFormat::Float32x4.size(),
                    shader_location: 4,
                },
            ],
        });
        descriptor.fragment.as_mut().unwrap().shader = self.shader.clone();

        Ok(descriptor)
    }
}

type DrawCustom = (
    SetItemPipeline,
    SetMeshViewBindGroup<0>,
    SetMeshBindGroup<1>,
    DrawMeshInstanced,
);

struct DrawMeshInstanced;
impl<P: PhaseItem> RenderCommand<P> for DrawMeshInstanced {
    type Param = SRes<RenderAssets<Mesh>>;
    type ViewWorldQuery = ();
    type ItemWorldQuery = (Read<Handle<Mesh>>, Read<InstanceBuffer>);
    #[inline]
    fn render<'w>(
        _item: &P,
        _view: (),
        (mesh_handle, instance_buffer): (&'w Handle<Mesh>, &'w InstanceBuffer),
        meshes: SystemParamItem<'w, '_, Self::Param>,
        pass: &mut TrackedRenderPass<'w>,
    ) -> RenderCommandResult {
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
            GpuBufferInfo::NonIndexed => {
                pass.draw(0..gpu_mesh.vertex_count, 0..instance_buffer.length as u32);
            }
        }
        RenderCommandResult::Success
    }
}
