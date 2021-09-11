use bevy::{
    prelude::*,
    render::{
        camera::PerspectiveProjection,
        pipeline::{PipelineDescriptor, RenderPipeline},
        shader::{ShaderStage, ShaderStages},
        wireframe::{Wireframe, WireframePlugin},
    },
    wgpu::{WgpuFeature, WgpuFeatures, WgpuOptions},
};

use simula_camera::orbitcam::*;
use simula_viz::{axes, line, voxel};

fn main() {
    App::new()
        .insert_resource(WindowDescriptor {
            title: "[Simbotic] Simula - Voxel".to_string(),
            vsync: false,
            ..Default::default()
        })
        .insert_resource(Msaa { samples: 4 })
        .insert_resource(WgpuOptions {
            features: WgpuFeatures {
                features: vec![WgpuFeature::NonFillPolygonMode],
            },
            ..Default::default()
        })
        .add_plugins(DefaultPlugins)
        .add_plugin(WireframePlugin)
        .add_plugin(OrbitCameraPlugin)
        .add_plugin(line::LinesPlugin)
        .insert_resource(line::Lines {
            depth_test: true,
            ..Default::default()
        })
        .add_system(axes::system)
        .add_startup_system(voxel::setup)
        .add_startup_system(setup)
        .run();
}

fn setup(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut pipelines: ResMut<Assets<PipelineDescriptor>>,
    mut shaders: ResMut<Assets<Shader>>,
) {
    let pipeline_handle = pipelines.add(PipelineDescriptor::default_config(ShaderStages {
        vertex: shaders.add(Shader::from_glsl(ShaderStage::Vertex, voxel::VERTEX_SHADER)),
        fragment: Some(shaders.add(Shader::from_glsl(
            ShaderStage::Fragment,
            voxel::FRAGMENT_SHADER,
        ))),
    }));

    let voxels: Vec<voxel::Voxel> = vec![
        voxel::Voxel {
            position: Vec3::new(10., 0., 0.),
            size: 1.,
            color: Color::RED,
        },
        voxel::Voxel {
            position: Vec3::new(0., 10., 0.),
            size: 1.,
            color: Color::GREEN,
        },
        voxel::Voxel {
            position: Vec3::new(0., 0., -10.),
            size: 1.,
            color: Color::BLUE,
        },
    ];

    let lattice = voxel::merge(voxels.clone());

    commands
        .spawn_bundle(MeshBundle {
            mesh: meshes.add(lattice),
            transform: Transform {
                ..Default::default()
            },
            render_pipelines: RenderPipelines::from_pipelines(vec![RenderPipeline::new(
                pipeline_handle.clone(),
            )]),
            ..Default::default()
        })
        .with_children(|parent| {
            parent.spawn_bundle(axes::AxesBundle {
                axes: axes::Axes { size: 10. },
                ..Default::default()
            });
        })
        .insert(Wireframe);

    commands
        .spawn_bundle(PerspectiveCameraBundle {
            transform: Transform::from_translation(Vec3::new(0.0, 0.0, 20.0))
                .looking_at(Vec3::default(), Vec3::Y),
            perspective_projection: PerspectiveProjection {
                near: 0.1,
                far: 10000.,
                ..Default::default()
            },
            ..Default::default()
        })
        .insert(OrbitCamera::default());
}
