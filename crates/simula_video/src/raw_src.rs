use bevy::{
    prelude::*,
    render::{
        camera::RenderTarget, main_graph::node::CAMERA_DRIVER, render_asset::RenderAssets,
        render_graph::RenderGraph, render_resource::*, renderer::RenderDevice, Extract, RenderApp,
        RenderSet,
    },
};
use crossbeam_channel::{bounded, Receiver, Sender};
use std::num::NonZeroU32;

#[derive(Component)]
pub struct RawSrc {
    pub data: Vec<u8>,
    pub size: UVec2,
}

#[derive(Component)]
pub struct RawBuffer {
    buffer: Buffer,
    sender: Sender<Vec<u8>>,
    receiver: Receiver<Vec<u8>>,
}

pub(crate) fn setup_raw_src() {}

pub(crate) fn setup_raw_srcs(
    mut commands: Commands,
    device: Res<RenderDevice>,
    images: Res<Assets<Image>>,
    srcs: Query<(Entity, &Camera, &RawSrc), (With<RawSrc>, Without<RawBuffer>)>,
) {
    for (entity, camera, src) in srcs.iter() {
        if let RenderTarget::Image(image) = &camera.target {
            if let Some(image) = images.get(image) {
                let size = image.size();

                if size.x as u32 != src.size.x || size.y as u32 != src.size.y {
                    error!("RawSrc size does not match Camera target size");
                }

                let padded_bytes_per_row =
                    RenderDevice::align_copy_bytes_per_row((size.x) as usize) * 4;

                let (sender, receiver) = bounded(1);

                commands.entity(entity).insert(RawBuffer {
                    buffer: device.create_buffer(&BufferDescriptor {
                        label: Some("raw_src_buffer"),
                        size: padded_bytes_per_row as u64 * size.y as u64,
                        usage: BufferUsages::COPY_DST | BufferUsages::MAP_READ,
                        mapped_at_creation: false,
                    }),
                    sender,
                    receiver,
                });
            }
        } else {
            panic!("GstSrc requires a camera with a RenderTarget::Image");
        }
    }
}

pub(crate) fn setup_render_graph(app: &mut App) {
    let render_app = app
        .sub_app_mut(RenderApp)
        .add_system(extract_raw_srcs.in_set(RenderSet::ExtractCommands))
        .add_system(cleanup_raw_srcs.in_set(RenderSet::Cleanup));

    let mut graph = render_app.world.get_resource_mut::<RenderGraph>().unwrap();

    graph.add_node(NODE_NAME, RawSrcNode::default());
    graph.add_node_edge(CAMERA_DRIVER, NODE_NAME);
}

pub(crate) fn process_raw_srcs(mut srcs: Query<(&RawBuffer, &mut RawSrc)>) {
    for (buffer, mut src) in srcs.iter_mut() {
        if let Ok(data) = buffer.receiver.try_recv() {
            src.data = data.clone();
        }
    }
}

#[derive(Component, Clone)]
pub struct RawSrcBlit {
    image: Handle<Image>,
    buffer: Buffer,
    sender: Sender<Vec<u8>>,
}

pub(crate) fn extract_raw_srcs(
    mut commands: Commands,
    images: Res<RenderAssets<Image>>,
    srcs: Extract<Query<(Entity, &RawBuffer, &Camera)>>,
) {
    for (entity, buffer, camera) in srcs.iter() {
        if let RenderTarget::Image(image) = &camera.target {
            if images.get(image).is_some() {
                commands.get_or_spawn(entity).insert(RawSrcBlit {
                    image: image.clone(),
                    buffer: buffer.buffer.clone(),
                    sender: buffer.sender.clone(),
                });
            }
        }
    }
}

pub(crate) fn cleanup_raw_srcs(
    mut commands: Commands,
    device: Res<RenderDevice>,
    srcs: Query<(Entity, &RawSrcBlit)>,
) {
    for (entity, src) in srcs.iter() {
        let data = {
            let slice = src.buffer.slice(..);
            {
                let (mapping_tx, mapping_rx) = bounded(1);
                device.map_buffer(&slice, MapMode::Read, move |res| {
                    mapping_tx.send(res).unwrap();
                });
                device.poll(wgpu::Maintain::Wait);
                let _ = mapping_rx.recv().unwrap();
            }
            slice.get_mapped_range().to_vec()
        };
        src.buffer.unmap();
        let _ = src.sender.try_send(data);
        commands.entity(entity).remove::<RawSrcBlit>();
    }
}

pub const NODE_NAME: &str = "raw_src_node";

#[derive(Default)]
pub struct RawSrcNode {
    srcs: Vec<RawSrcBlit>,
}

impl bevy::render::render_graph::Node for RawSrcNode {
    fn update(&mut self, world: &mut World) {
        self.srcs = world.query::<&RawSrcBlit>().iter(world).cloned().collect();
    }

    fn run(
        &self,
        _graph: &mut bevy::render::render_graph::RenderGraphContext,
        render_context: &mut bevy::render::renderer::RenderContext,
        world: &World,
    ) -> Result<(), bevy::render::render_graph::NodeRunError> {
        for src in &self.srcs {
            if let Some(src_image) = world
                .get_resource::<RenderAssets<Image>>()
                .unwrap()
                .get(&src.image)
            {
                let size = src_image.size;

                let format = src_image.texture_format.describe();

                let padded_bytes_per_row = RenderDevice::align_copy_bytes_per_row(
                    (size.x as usize / format.block_dimensions.0 as usize)
                        * format.block_size as usize,
                );

                render_context.command_encoder().copy_texture_to_buffer(
                    src_image.texture.as_image_copy(),
                    ImageCopyBuffer {
                        buffer: &src.buffer,
                        layout: ImageDataLayout {
                            offset: 0,
                            bytes_per_row: Some(
                                NonZeroU32::new(padded_bytes_per_row as u32).unwrap(),
                            ),
                            rows_per_image: Some(NonZeroU32::new(size.y as u32).unwrap()),
                        },
                    },
                    Extent3d {
                        width: size.x as u32,
                        height: size.y as u32,
                        ..default()
                    },
                );
            }
        }

        Ok(())
    }
}
