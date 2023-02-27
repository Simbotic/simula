use bevy::prelude::*;
use serde::{Deserialize, Serialize};
use simula_behavior::prelude::*;

use crate::{
    behavior_trees::robber::robber_captured::RobberCaptured,
    behaviors::{
        movement::{Movement, RobotMove},
        rest::RobotRest,
    },
    common::Robot,
    components::{cop::*, robber::*},
};

use super::cop_alerted::CopAlerted;

#[derive(Debug, Default, Component, Reflect, Clone, Deserialize, Serialize)]
pub struct CopCaptureAction;

impl BehaviorInfo for CopCaptureAction {
    const TYPE: BehaviorType = BehaviorType::Action;
    const NAME: &'static str = "CopCaptureAction";
    const DESC: &'static str =
        "Capture the Robber. The Robber loses all of their energy and money.";
}

pub const COP_CAPTURE_ENERGY_COST: f32 = COP_STARTING_ENERGY / 5.0;
pub const COP_CAPTURE_RADIUS: f32 = 0.5;

pub fn run(
    mut commands: Commands,
    action_query: Query<(Entity, &CopCaptureAction, &BehaviorNode), BehaviorRunQuery>,
    mut query: Query<(&Transform, &mut Cop, Option<&CopAlerted>, &mut Movement)>,
    mut robber_query: Query<(Entity, &Transform, &mut Robber), Without<RobberCaptured>>,
) {
    for (action_entity, _, node) in &action_query {
        if let Some(cop_entity) = node.tree {
            if let Ok((cop_transform, mut cop, alerted, mut movement)) = query.get_mut(cop_entity) {
                for (robber_entity, robber_transform, mut robber) in robber_query.iter_mut() {
                    let cop_money = cop.get_money();
                    let cop_energy = cop.get_energy();
                    if cop_energy < COP_CAPTURE_ENERGY_COST {
                        commands.entity(action_entity).insert(BehaviorSuccess);
                        return;
                    }
                    let distance =
                        (robber_transform.translation - cop_transform.translation).length();
                    if distance <= COP_CAPTURE_RADIUS {
                        let robber_money = robber.get_money();
                        cop.set_energy(cop_energy - COP_CAPTURE_ENERGY_COST);
                        robber.set_money(0);
                        robber.set_energy(0.0);
                        cop.set_money(cop_money + robber_money);
                        commands
                            .entity(robber_entity)
                            .remove::<RobotMove>()
                            .remove::<RobotRest>()
                            .insert(RobberCaptured::new());
                        info!(
                            "[Cop {:?}] Captured Robber {:?} using {} energy",
                            cop_entity, robber_entity, COP_CAPTURE_ENERGY_COST
                        );
                        info!(
                            "[Robber {:?}] Captured by Cop {:?} with {} money",
                            robber_entity, cop_entity, robber_money
                        );
                        if alerted.is_some() {
                            commands.entity(cop_entity).remove::<CopAlerted>();
                            movement.duration = COP_SPEED;
                            info!(
                                "[Cop {:?}] Captured Robber {:?} and is no longer alerted",
                                cop_entity, robber_entity
                            )
                        }
                        commands.entity(action_entity).insert(BehaviorSuccess);
                    }
                }
            }
        }
    }
}
