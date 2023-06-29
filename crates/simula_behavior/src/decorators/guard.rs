use crate::prelude::*;
use bevy::prelude::*;
use serde::{Deserialize, Serialize};
use simula_script::{Script, ScriptContext};

/// Guard evals a script to control the flow of execution. If the script returns
/// `true`, the child is executed. If the script returns `false`, the child is
/// not executed. The Scope of the script should be at the tree entity.
#[derive(
    Debug, Deref, DerefMut, Component, Reflect, FromReflect, Clone, Deserialize, Serialize,
)]
pub struct Guard(BehaviorEval<bool>);

impl Default for Guard {
    fn default() -> Self {
        Self(BehaviorEval {
            eval: "true".into(),
            result: None,
            script_handle: None,
        })
    }
}

impl BehaviorInfo for Guard {
    const TYPE: BehaviorType = BehaviorType::Decorator;
    const NAME: &'static str = "Guard";
    const ICON: &'static str = "🚫";
    const DESC: &'static str =
        "Guard evals a script to control the flow of execution. If the script returns \
        `true`, the child is executed. If the script returns `false`, the child is \
        not executed. The Scope of the script should be at the tree entity.";
}

pub fn run(
    mut commands: Commands,
    mut guards: Query<
        (
            Entity,
            &BehaviorChildren,
            &mut Guard,
            &BehaviorNode,
            Option<&BehaviorStarted>,
        ),
        BehaviorRunQuery,
    >,
    nodes: Query<BehaviorChildQuery, BehaviorChildQueryFilter>,
    mut scripts: ResMut<Assets<Script>>,
    script_ctx_handles: Query<&Handle<ScriptContext>>,
    mut script_ctxs: ResMut<Assets<ScriptContext>>,
) {
    for (entity, children, mut guard, node, started) in &mut guards {
        if children.len() != 1 {
            error!("Decorator node requires one child");
            commands.entity(entity).insert(BehaviorFailure);
            continue;
        }

        if started.is_some() {
            guard.result = None;
        }

        if guard.script_handle.is_none() {
            if let Err(err) =
                guard.make_handle(node, &mut scripts, &script_ctx_handles, &mut script_ctxs)
            {
                error!("Script errored: {:?}", err);
                commands.entity(entity).insert(BehaviorFailure);
            }
        } else {
            let child_entity = children[0]; // Safe because we checked for empty
            if let Ok(BehaviorChildQueryItem {
                child_entity,
                child_parent: _,
                child_failure,
                child_success,
                child_running: _,
            }) = nodes.get(child_entity)
            {
                // Child failed, we fail
                if child_failure.is_some() {
                    commands.entity(entity).insert(BehaviorFailure);
                }
                // Child succeeded, so we succeed
                else if child_success.is_some() {
                    commands.entity(entity).insert(BehaviorSuccess);
                }
                // Child is ready, eval script to see if we should pass on cursor
                else {
                    if guard.script_handle.is_some() {
                        let result = guard.eval(
                            node,
                            scripts.as_ref(),
                            &script_ctx_handles,
                            &mut script_ctxs,
                        );
                        if let Err(err) = result {
                            error!("Script errored: {:?}", err);
                            commands.entity(entity).insert(BehaviorFailure);
                            continue;
                        }
                        match &guard.result {
                            Some(true) => {
                                // Script returned true, so let the child run
                                commands.entity(entity).remove::<BehaviorCursor>();
                                commands
                                    .entity(child_entity)
                                    .insert(BehaviorCursor::Delegate);
                            }
                            Some(false) => {
                                // Script returned false, so we fail
                                commands.entity(entity).insert(BehaviorFailure);
                            }
                            None => {
                                // Script is still running, so we keep running
                            }
                        };
                    } else {
                        // Script is still loading, so we keep running
                    }
                }
            }
        }
    }
}
