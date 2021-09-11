// use crate::{lattice::Lattice, viz};
use bevy::{
    prelude::*,
    render::{
        pipeline::{PipelineDescriptor, RenderPipeline},
        shader::{ShaderStage, ShaderStages},
    },
};
use ndarray::prelude::*;
use simula_core::lattice::Lattice;
use simula_viz::{axes, voxel};

pub struct WorldData {
    pub color: Color,
}

pub struct World {
    pub array: <Self as Lattice>::Array,
}

impl World {
    pub fn new() -> World {
        let array =
            <Self as Lattice>::Array::zeros((Self::SIZE, Self::SIZE, Self::SIZE, Self::ELEMENTS));
        World { array }
    }
}

impl Lattice for World {
    type Array = Array4<f32>;
    type Value = WorldData;
    const SIZE: usize = 16;
    const ELEMENTS: usize = 4;

    fn array(&self) -> &Self::Array {
        &self.array
    }

    fn array_(&mut self) -> &mut Self::Array {
        &mut self.array
    }

    fn set(&mut self, x: usize, y: usize, z: usize, value: WorldData) {
        self.array_()[[x, y, z, 0]] = value.color.r();
        self.array_()[[x, y, z, 1]] = value.color.g();
        self.array_()[[x, y, z, 2]] = value.color.b();
        self.array_()[[x, y, z, 3]] = value.color.a();
    }

    fn get(&self, x: usize, y: usize, z: usize) -> WorldData {
        let color = Color::Rgba {
            red: self.array()[[x, y, z, 0]],
            green: self.array()[[x, y, z, 1]],
            blue: self.array()[[x, y, z, 2]],
            alpha: self.array()[[x, y, z, 3]],
        };
        WorldData { color }
    }
}

pub fn setup(
    mut commands: Commands,
    mut world: ResMut<World>,
    mut meshes: ResMut<Assets<Mesh>>,
    mut pipelines: ResMut<Assets<PipelineDescriptor>>,
    mut shaders: ResMut<Assets<Shader>>,
) {
    world.set(0, 0, 0, WorldData { color: Color::BLUE });
    world.set(8, 8, 8, WorldData { color: Color::RED });
    world.set(15, 15, 15, WorldData { color: Color::CYAN });

    let mut voxels = Vec::with_capacity(16 * 16 * 16);

    let mut color = [0f32; 4];
    for ((x, y, z, c), i) in world.array().indexed_iter() {
        color[c] = *i;
        if c == 3 && *i > 0. {
            voxels.push(voxel::Voxel {
                position: Vec3::new(x as f32, y as f32, z as f32),
                size: 1.,
                color: Color::rgba(color[0], color[1], color[2], color[3]),
            });
        }
    }

    let lattice = voxel::merge(voxels);

    let pipeline_handle = pipelines.add(PipelineDescriptor::default_config(ShaderStages {
        vertex: shaders.add(Shader::from_glsl(ShaderStage::Vertex, voxel::VERTEX_SHADER)),
        fragment: Some(shaders.add(Shader::from_glsl(
            ShaderStage::Fragment,
            voxel::FRAGMENT_SHADER,
        ))),
    }));

    commands
        .spawn_bundle(MeshBundle {
            mesh: meshes.add(lattice),
            transform: Transform {
                scale: Vec3::new(1., 1., 1.),
                ..Default::default()
            },
            render_pipelines: RenderPipelines::from_pipelines(vec![RenderPipeline::new(
                pipeline_handle.clone(),
            )]),
            ..Default::default()
        })
        .with_children(|parent| {
            parent.spawn_bundle(axes::AxesBundle {
                axes: axes::Axes { size: 1. },
                ..Default::default()
            });
        });
}

// pub fn system(_world: Res<World>) {}
