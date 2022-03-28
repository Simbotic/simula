use crate::{
    axes::{Axes, AxesBundle, AxesPlugin},
    grid::{Grid, GridBundle, GridPlugin},
    lines::{Lines, LinesBundle, LinesMaterial, LinesPlugin},
    voxels::{Voxel, Voxels, VoxelsBundle, VoxelsPlugin},
};
use bevy::prelude::*;
use simula_core::{force_graph, signal};

// #[derive(Component, Clone, Default)]
#[derive(Reflect, Component)]
#[reflect(Component)]
pub struct ForceGraph<
    UserNodeData: Default + Reflect + PartialEq,
    UserEdgeData: Default + Reflect + PartialEq,
> {
    #[reflect(ignore)]
    pub graph: force_graph::ForceGraph<UserNodeData, UserEdgeData>,
    pub parameters: force_graph::SimulationParameters,
}

impl<UserNodeData: Default + Reflect + PartialEq, UserEdgeData: Default + Reflect + PartialEq>
    Default for ForceGraph<UserNodeData, UserEdgeData>
{
    fn default() -> Self {
        let parameters = force_graph::SimulationParameters {
            force_charge: 1.2,
            force_spring: 0.0003,
            force_max: 0.0280,
            node_speed: 0.0, //0.7,
            damping_factor: 0.95,
        };

        ForceGraph {
            graph: force_graph::ForceGraph::new(parameters.clone()),
            parameters,
        }
    }
}

#[derive(Bundle)]
pub struct ForceGraphBundle<
    UserNodeData: Default + Reflect + PartialEq,
    UserEdgeData: Default + Reflect + PartialEq,
> {
    pub graph: ForceGraph<UserNodeData, UserEdgeData>,
    pub lines: Lines,
    pub material: LinesMaterial,
    pub transform: Transform,
    pub global_transform: GlobalTransform,
    pub visibility: Visibility,
    pub computed_visibility: ComputedVisibility,
}
