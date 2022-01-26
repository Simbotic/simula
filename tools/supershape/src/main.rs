use bevy::{
    prelude::*,
    render::{
        camera::PerspectiveProjection,
        pipeline::{PipelineDescriptor, RenderPipeline},
        shader::{ShaderStage, ShaderStages},
    },
    wgpu::{WgpuFeature, WgpuFeatures, WgpuOptions},
};

use simula_camera::orbitcam::*;
use simula_viz::{axes, grid, lines, voxel};

fn main() {
    App::new()
        .insert_resource(WindowDescriptor {
            title: "[Simbotic] Simula - Supershape".to_string(),
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
        .insert_resource(ClearColor(Color::rgb(0.125, 0.12, 0.13)))
        .add_plugins(DefaultPlugins)
        .add_plugin(OrbitCameraPlugin)
        .add_plugin(lines::LinesPlugin)
        .insert_resource(lines::Lines {
            depth_test: true,
            ..Default::default()
        })
        .add_system(axes::system)
        .add_startup_system(setup)
        .add_system(grid::system)
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
            position: Vec3::new(5., 0., 0.),
            size: 0.1,
            color: Color::RED,
        },
        voxel::Voxel {
            position: Vec3::new(0., 5., 0.),
            size: 0.1,
            color: Color::GREEN,
        },
        voxel::Voxel {
            position: Vec3::new(0., 0., -5.),
            size: 0.1,
            color: Color::BLUE,
        },
    ];

    let lattice = voxel::merge(voxels.clone());

    commands.spawn_bundle(grid::GridBundle::default());

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
                axes: axes::Axes { size: 1., inner_offset: 5. },
                ..Default::default()
            });
        });

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
