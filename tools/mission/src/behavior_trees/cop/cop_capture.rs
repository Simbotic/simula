use bevy::prelude::*;
use serde::{Deserialize, Serialize};
use simula_behavior::prelude::*;

use crate::{
    common::{CanRotate, Robot},
    cop::{Cop, CopChase},
    robber::{Robber, RobberCaptured, RobberRest, RobberRun},
};

#[derive(Debug, Default, Component, Reflect, Clone, Deserialize, Serialize, Inspectable)]
pub struct CopCaptureAction;

impl BehaviorInfo for CopCaptureAction {
    const TYPE: BehaviorType = BehaviorType::Action;
    const NAME: &'static str = "CopCaptureAction";
    const DESC: &'static str =
        "Capture the Robber. The Robber loses all of their energy and money.";
}

pub const COP_CAPTURE_ENERGY_COST: u64 = 100;
pub const COP_CAPTURE_RANGE: f32 = 0.25;

pub fn run(
    mut commands: Commands,
    action_query: Query<(Entity, &CopCaptureAction, &BehaviorNode), BehaviorRunQuery>,
    mut query: Query<(&Transform, &mut Cop), With<CopChase>>,
    mut robber_query: Query<(Entity, &Transform, &mut Robber), Without<RobberCaptured>>,
) {
    for (action_entity, _, node) in &action_query {
        if let Some(cop_entity) = node.tree {
            if let Ok((cop_transform, mut cop)) = query.get_mut(cop_entity) {
                for (robber_entity, robber_transform, mut robber) in robber_query.iter_mut() {
                    let cop_money = cop.get_money();
                    let cop_energy = cop.get_energy();
                    if cop_energy < COP_CAPTURE_ENERGY_COST {
                        commands.entity(action_entity).insert(BehaviorFailure);
                        return;
                    }
                    let robber_rotation = robber_transform.rotation;
                    let cop_rotation = cop_transform.rotation;
                    let rotation_diff = robber_rotation.y - cop_rotation.y;
                    if rotation_diff < COP_CAPTURE_RANGE && rotation_diff > -COP_CAPTURE_RANGE {
                        let robber_money = robber.get_money();
                        cop.set_energy(cop_energy - COP_CAPTURE_ENERGY_COST);
                        robber.set_money(0);
                        robber.set_energy(0);
                        cop.set_money(cop_money + robber_money);
                        commands
                            .entity(robber_entity)
                            .remove::<CanRotate>()
                            .remove::<RobberRun>()
                            .remove::<RobberRest>()
                            .insert(RobberCaptured);
                        info!(
                            "[Cop {:?}] Captured Robber {:?} using {} energy",
                            cop_entity, robber_entity, COP_CAPTURE_ENERGY_COST
                        );
                        info!(
                            "[Robber {:?}] Captured by Cop {:?} with {} money",
                            robber_entity, cop_entity, robber_money
                        );
                    }
                }
            }
        }
        commands.entity(action_entity).insert(BehaviorSuccess);
    }
}
