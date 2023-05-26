use crate::prelude::*;
use bevy::prelude::*;
use serde::{Deserialize, Serialize};
use simula_script::{Scope, Script};

#[derive(Debug, Reflect, FromReflect, Clone, Deserialize, Serialize)]
pub enum Source {
    Asset(String),
    Inline(String),
}

/// Guard evals a script to control the flow of execution. If the script returns
/// `true`, the child is executed. If the script returns `false`, the child is
/// not executed. The Scope of the script should be at the tree entity.
#[derive(Debug, Component, Reflect, Clone, Deserialize, Serialize)]
pub struct Guard {
    /// The script to evaluate
    pub script: Source,
}

impl Default for Guard {
    fn default() -> Self {
        Self {
            script: Source::Inline("true".into()),
        }
    }
}

impl BehaviorInfo for Guard {
    const TYPE: BehaviorType = BehaviorType::Decorator;
    const NAME: &'static str = "Guard";
    const DESC: &'static str = "Guard evals script to control the flow of execution.";
}

pub fn run(
    mut commands: Commands,
    mut gates: Query<
        (
            Entity,
            &BehaviorChildren,
            &Guard,
            Option<&Handle<Script>>,
            &BehaviorNode,
        ),
        BehaviorRunQuery,
    >,
    nodes: Query<BehaviorChildQuery, BehaviorChildQueryFilter>,
    scope_handles: Query<&Handle<Scope>>,
    asset_server: ResMut<AssetServer>,
    mut scripts: ResMut<Assets<Script>>,
    mut scopes: ResMut<Assets<Scope>>,
) {
    for (entity, children, gate, script_handle, node) in &mut gates {
        if children.len() != 1 {
            error!("Decorator node requires one child");
            commands.entity(entity).insert(BehaviorFailure);
            continue;
        }

        if script_handle.is_none() {
            let script_handle = match &gate.script {
                Source::Asset(path) => asset_server.load(path),
                Source::Inline(script) => {
                    let script_asset = Script::from_str(script);
                    match script_asset {
                        Ok(script_asset) => scripts.add(script_asset),
                        Err(err) => {
                            error!("Script errored: {:?}", err);
                            commands.entity(entity).insert(BehaviorFailure);
                            continue;
                        }
                    }
                }
            };
            commands.entity(entity).insert(script_handle);
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
                    if let Some(script_asset) =
                        script_handle.and_then(|script_handle| scripts.get(script_handle))
                    {
                        if let Some(scope) = node.tree.and_then(|tree| scope_handles.get(tree).ok())
                        {
                            if let Some(scope) = scopes.get_mut(&scope) {
                                let result = script_asset.eval::<bool>(scope);
                                match result {
                                    Ok(true) => {
                                        // Script returned true, so let the child run
                                        commands.entity(entity).remove::<BehaviorCursor>();
                                        commands
                                            .entity(child_entity)
                                            .insert(BehaviorCursor::Delegate);
                                    }
                                    Ok(false) => {
                                        // Script returned false, so we fail
                                        commands.entity(entity).insert(BehaviorFailure);
                                    }
                                    Err(err) => {
                                        // Script errored, so we fail
                                        error!("Script errored: {:?}", err);
                                        commands.entity(entity).insert(BehaviorFailure);
                                    }
                                };
                            } else {
                                error!("Invalid scope handle");
                                commands.entity(entity).insert(BehaviorFailure);
                            }
                        } else {
                            error!("Cannot find scope handle in tree entity");
                            commands.entity(entity).insert(BehaviorFailure);
                        };
                    } else {
                        // TODO: Revisit to consider loading times
                        warn!("Script asset not loaded");
                        commands.entity(entity).insert(BehaviorFailure);
                    }
                }
            }
        }
    }
}
