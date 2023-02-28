use crate::lines::Lines;
use bevy::prelude::*;
use simula_core::force_graph;

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
            force_charge: 1200.0,
            force_spring: 1000.0,
            force_max: 2000.0,
            node_speed: Vec3::new(100.0, 0.0, 100.0),
            damping_factor: 0.95,
            iterations: 3,
            ..Default::default()
        };

        ForceGraph {
            graph: force_graph::ForceGraph::new(parameters.clone()),
            parameters,
        }
    }
}

#[derive(Bundle, Default)]
pub struct ForceGraphBundle<
    UserNodeData: Default + Reflect + PartialEq,
    UserEdgeData: Default + Reflect + PartialEq,
> {
    pub graph: ForceGraph<UserNodeData, UserEdgeData>,
    pub lines: Lines,
    pub transform: Transform,
    pub global_transform: GlobalTransform,
    pub visibility: Visibility,
    pub computed_visibility: ComputedVisibility,
}
