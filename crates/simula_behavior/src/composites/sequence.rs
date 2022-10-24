use crate::{
    BehaviorBundle, BehaviorChildren, BehaviorCursor, BehaviorInfo, BehaviorParent,
    BehaviorRunQuery, BehaviorRunning, BehaviorSuccess, BehaviorType, BehaviorWithoutQuery,
};
use bevy::{ecs::system::EntityCommands, prelude::*};

#[derive(Debug, Default, Component, Reflect, Clone)]
pub struct Sequence;

impl BehaviorInfo for Sequence {
    const TYPE: BehaviorType = BehaviorType::Action;
    const NAME: &'static str = "Sequence";
    const DESC: &'static str = "Sequence behavior node";

    fn spawn(commands: &mut EntityCommands) {
        commands
            .insert_bundle(BehaviorBundle::<Self>::default())
            .insert(BehaviorRunning)
            .insert(BehaviorCursor);
    }
}

pub fn run(
    mut commands: Commands,
    mut sequences: Query<(Entity, &BehaviorChildren), (With<Sequence>, BehaviorRunQuery)>,
    nodes: Query<(Entity, &BehaviorParent), BehaviorWithoutQuery>,
) {
    for (entity, children) in &mut sequences {
        if children.children.is_empty() {
            commands.entity(entity).insert(BehaviorSuccess);
        } else {
            let mut done = true;
            for (child_entity, child_parent) in nodes.iter_many(&children.children) {
                if let Some(child_parent) = child_parent.parent {
                    if entity == child_parent {
                        done = false;
                        commands.entity(entity).remove::<BehaviorCursor>();
                        commands.entity(child_entity).insert(BehaviorCursor);
                        commands.entity(child_entity).insert(BehaviorRunning);
                        break;
                    }
                }
            }
            if done {
                commands.entity(entity).insert(BehaviorSuccess);
            }
        }
    }
}
