use crate::prelude::*;
use bevy::prelude::*;
use rand::{rngs::StdRng, seq::SliceRandom, SeedableRng};
use serde::{Deserialize, Serialize};

/// A selector will return a success if any of its children succeed and not process
/// any further children. It will process the first child, and if it fails will
/// process the second, until a success is reached, at which point it will instantly
/// return success. It will fail if all children fail.
#[derive(Debug, Component, Reflect, Clone, Deserialize, Serialize)]
pub struct Selector {
    #[serde(default)]
    random: bool,
    #[serde(skip)]
    pub seed: u64,
}

impl Default for Selector {
    fn default() -> Self {
        Self {
            random: false,
            seed: rand::random(),
        }
    }
}

impl BehaviorInfo for Selector {
    const TYPE: BehaviorType = BehaviorType::Composite;
    const NAME: &'static str = "? Selector";
    const DESC: &'static str = "Selector behavior node";
}

pub fn run(
    mut commands: Commands,
    mut selectors: Query<
        (Entity, &BehaviorChildren, &mut Selector),
        (With<Selector>, BehaviorRunQuery),
    >,
    nodes: Query<BehaviorChildQuery, BehaviorChildQueryFilter>,
) {
    for (entity, children, mut selector) in &mut selectors {
        // If selector is random, shuffle the children, deterministically
        let mut rng = StdRng::seed_from_u64(selector.seed);
        let mut random_children;
        let children_iter = if selector.random {
            random_children = children.0.clone();
            random_children.shuffle(&mut rng);
            random_children.iter()
        } else {
            children.iter()
        };

        if children.is_empty() {
            commands.entity(entity).insert(BehaviorSuccess);
        } else {
            let mut should_fail = true;
            for BehaviorChildQueryItem {
                child_entity,
                child_parent: _,
                child_failure,
                child_success,
                child_running: _,
            } in nodes.iter_many(children_iter)
            {
                if child_failure.is_some() {
                    // Child failed, so we move to next child
                } else if child_success.is_some() {
                    // Child succeeded, so we succeed
                    commands.entity(entity).insert(BehaviorSuccess);
                    if selector.random {
                        selector.seed = rand::random();
                    }
                    should_fail = false;
                    break;
                } else {
                    // Child is ready, pass on cursor
                    commands.entity(entity).remove::<BehaviorCursor>();
                    commands
                        .entity(child_entity)
                        .insert(BehaviorCursor::Delegate);
                    should_fail = false;
                    break;
                }
            }
            // If all children failed, complete with failure
            if should_fail {
                commands.entity(entity).insert(BehaviorFailure);
                if selector.random {
                    selector.seed = rand::random();
                }
            }
        }
    }
}
