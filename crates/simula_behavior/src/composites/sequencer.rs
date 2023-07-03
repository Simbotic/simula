use crate::prelude::*;
use bevy::prelude::*;
use rand::{rngs::StdRng, seq::SliceRandom, SeedableRng};
use serde::{Deserialize, Serialize};

/// A sequence will visit each child in order, starting with the first, and when that
/// succeeds will call the second, and so on down the list of children. If any child
/// fails it will immediately return failure to the parent. If the last child in the
/// sequence succeeds, then the sequence will return success to its parent.
#[derive(Debug, Component, Reflect, FromReflect, Clone, Deserialize, Serialize)]
pub struct Sequencer {
    #[serde(default)]
    random: bool,
    #[serde(skip)]
    #[reflect(ignore)]
    pub seed: u64,
}

impl Default for Sequencer {
    fn default() -> Self {
        Self {
            random: false,
            seed: rand::random(),
        }
    }
}

impl BehaviorSpec for Sequencer {
    const TYPE: BehaviorType = BehaviorType::Composite;
    const NAME: &'static str = "Sequencer";
    const ICON: &'static str = "➡";
    const DESC: &'static str =
        "A Sequence will visit each child in order, starting with the first, and when that \
        succeeds will call the second, and so on down the list of children. If any child \
        fails it will immediately return failure to the parent. If the last child in the \
        sequence succeeds, then the sequence will return success to its parent.";
}

impl BehaviorUI for Sequencer {}

pub fn run(
    mut commands: Commands,
    mut sequences: Query<
        (Entity, &BehaviorChildren, &mut Sequencer),
        (With<Sequencer>, BehaviorRunQuery),
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
                child_parent: _,
                child_failure,
                child_success,
                child_running: _,
            } in nodes.iter_many(children_iter)
            {
                if child_failure.is_some() {
                    // Child failed, so we fail
                    commands.entity(entity).insert(BehaviorFailure);
                    if sequence.random {
                        sequence.seed = rand::random();
                    }
                    should_succeed = false;
                    break;
                } else if child_success.is_some() {
                    // Child succeeded, so we move to next child
                } else {
                    // Child is ready, pass on cursor
                    commands.entity(entity).remove::<BehaviorCursor>();
                    commands
                        .entity(child_entity)
                        .insert(BehaviorCursor::Delegate);
                    should_succeed = false;
                    break;
                }
            }
            // If all children succeed, complete with success
            if should_succeed {
                commands.entity(entity).insert(BehaviorSuccess);
                if sequence.random {
                    sequence.seed = rand::random();
                }
            }
        }
    }
}
