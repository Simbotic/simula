use crate::prelude::*;
use bevy::prelude::*;
use rand::{rngs::StdRng, seq::SliceRandom, SeedableRng};
use serde::{Deserialize, Serialize};

/// A sequence will visit each child in order, starting with the first, and when that
/// succeeds will call the second, and so on down the list of children. If any child
/// fails it will immediately return failure to the parent. If the last child in the
/// sequence succeeds, then the sequence will return success to its parent.
#[derive(Debug, Component, Reflect, Clone, Deserialize, Serialize, Inspectable)]
pub struct Sequence {
    #[serde(default)]
    random: bool,
    #[serde(default)]
    pub seed: u64,
}

impl Default for Sequence {
    fn default() -> Self {
        Self {
            random: false,
            seed: rand::random(),
        }
    }
}

impl BehaviorInfo for Sequence {
    const TYPE: BehaviorType = BehaviorType::Composite;
    const NAME: &'static str = "Sequence";
    const DESC: &'static str = "Sequence behavior node";
}

pub fn run(
    mut commands: Commands,
    mut sequences: Query<
        (Entity, &BehaviorChildren, &mut Sequence),
        (With<Sequence>, BehaviorRunQuery),
    >,
    nodes: Query<BehaviorChildQuery, BehaviorChildQueryFilter>,
) {
    for (entity, children, mut sequence) in &mut sequences {
        // If sequence is random, shuffle the children, deterministically
        let mut rng = StdRng::seed_from_u64(sequence.seed);
        let mut random_children;
        let children_iter = if sequence.random {
            random_children = children.0.clone();
            random_children.shuffle(&mut rng);
            random_children.iter()
        } else {
            children.iter()
        };

        if children.is_empty() {
            commands.entity(entity).insert(BehaviorSuccess);
        } else {
            let mut should_succeed = true;
            for BehaviorChildQueryItem {
                child_entity,
                child_parent,
                child_failure,
                child_success,
            } in nodes.iter_many(children_iter)
            {
                if let Some(child_parent) = **child_parent {
                    if entity == child_parent {
                        if child_failure.is_some() {
                            // Child failed, so we fail
                            commands.entity(entity).insert(BehaviorFailure);
                            sequence.seed = rand::random();
                            should_succeed = false;
                            break;
                        } else if child_success.is_some() {
                            // Child succeeded, so we move to next child
                        } else {
                            // Child is ready, pass on cursor
                            commands.entity(entity).remove::<BehaviorCursor>();
                            commands.entity(child_entity).insert(BehaviorCursor);
                            should_succeed = false;
                            break;
                        }
                    } else {
                        // Child is not ours, so we fail
                        warn!("Child is not ours");
                        commands.entity(entity).insert(BehaviorFailure);
                        should_succeed = false;
                        break;
                    }
                } else {
                    // Child has no parent, so we fail
                    warn!("Child has no parent, so we fail");
                    commands.entity(entity).insert(BehaviorFailure);
                    should_succeed = false;
                    break;
                }
            }
            // If all children succeed, complete with success
            if should_succeed {
                commands.entity(entity).insert(BehaviorSuccess);
                sequence.seed = rand::random();
            }
        }
    }
}
