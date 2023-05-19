use crate::prelude::*;
use bevy::prelude::*;
use serde::{Deserialize, Serialize};
use simula_script::asset;
use std::fmt::Debug;

#[derive(Debug, Reflect, FromReflect, Clone, Deserialize, Serialize)]
pub enum Source {
    Asset(String),
    Inline(String),
}

/// Script for running complex logic before and after child execution.
#[derive(Debug, Default, Component, Reflect, Clone, Deserialize, Serialize)]
pub struct Script {
    /// Source of the script.
    pub source: Option<Source>,
}

impl BehaviorInfo for Script {
    const TYPE: BehaviorType = BehaviorType::Decorator;
    const NAME: &'static str = "Script";
    const DESC: &'static str = "Runs complex logic before and after child execution.";
}

pub fn run<T: BehaviorSpawner>(
    mut commands: Commands,
    mut scripts: Query<
        (
            Entity,
            &BehaviorChildren,
            &Script,
            Option<&Handle<asset::RhaiScript>>,
        ),
        BehaviorRunQuery,
    >,
    nodes: Query<BehaviorChildQuery, BehaviorChildQueryFilter>,
    mut script_assets: ResMut<Assets<asset::RhaiScript>>,
    asset_server: ResMut<AssetServer>,
) {
    for (entity, children, script, script_handle) in &mut scripts {
        if script_handle.is_none() {
            let script_handle = match &script.source {
                Some(Source::Asset(path)) => asset_server.load(path),
                Some(Source::Inline(script)) => {
                    let script_asset = asset::from_str(script);
                    script_assets.add(script_asset)
                }
                None => panic!("Script source is not set"),
            };
            commands.entity(entity).insert(script_handle);
        } else if children.is_empty() {
        } else {
            if children.len() > 1 {
                panic!("Decorator node has more than one child");
            }
            let child_entity = children[0]; // Safe because we checked for empty
            if let Ok(BehaviorChildQueryItem {
                child_entity,
                child_parent,
                child_failure,
                child_success,
                child_running: _,
            }) = nodes.get(child_entity)
            {
                if let Some(child_parent) = **child_parent {
                    if entity == child_parent {
                        // Child failed, we fail
                        if child_failure.is_some() {
                            commands.entity(entity).insert(BehaviorFailure);
                        }
                        // Child succeeded, so we succeed
                        else if child_success.is_some() {
                            commands.entity(entity).insert(BehaviorSuccess);
                        }
                        // Child is ready, pass on cursor
                        else {
                            commands.entity(entity).remove::<BehaviorCursor>();
                            commands.entity(child_entity).insert(BehaviorCursor);
                        }
                    } else {
                        // Child is not ours, so we fail
                        warn!("Child is not ours");
                        commands.entity(entity).insert(BehaviorFailure);
                    }
                } else {
                    // Child has no parent, so we fail
                    warn!("Child has no parent");
                    commands.entity(entity).insert(BehaviorFailure);
                }
            }
        }
    }
}
