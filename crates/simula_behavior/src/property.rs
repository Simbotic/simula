use crate::prelude::*;
use bevy::prelude::*;
use serde::{Deserialize, Serialize};
use simula_script::{Script, ScriptContext};
use std::borrow::Cow;

#[derive(Debug, Reflect, FromReflect, Clone, Deserialize, Serialize)]
pub enum BehaviorEval<T: Reflect + Default> {
    Value(T),
    Eval {
        eval: Cow<'static, str>,
        #[serde(skip)]
        #[reflect(ignore)]
        handle: Option<Handle<Script>>,
    },
}

impl<T: Reflect + Default> Default for BehaviorEval<T> {
    fn default() -> Self {
        Self::Value(T::default())
    }
}

#[derive(Debug, Reflect, FromReflect, Clone, Deserialize, Serialize, Default)]
pub struct BehaviorProperty<T: Reflect + Default> {
    pub prop: BehaviorEval<T>,
    #[serde(skip)]
    pub value: Option<T>,
}

impl<T> BehaviorProperty<T>
where
    T: Reflect + Default + Clone + for<'de> Deserialize<'de>,
{
    pub fn fetch(
        &mut self,
        node: &BehaviorNode,
        scripts: &mut ResMut<Assets<Script>>,
        script_ctx_handles: &Query<&Handle<ScriptContext>>,
        script_ctxs: &mut Assets<ScriptContext>,
    ) -> Option<Result<(), String>> {
        let state = match &mut self.prop {
            BehaviorEval::Eval { eval, handle } => {
                if handle.is_none() {
                    match make_handle(
                        eval.to_owned(),
                        node,
                        scripts,
                        script_ctx_handles,
                        script_ctxs,
                    ) {
                        Ok(new_handle) => {
                            *handle = Some(new_handle);
                            Ok(())
                        }
                        Err(err) => return Some(Err(err)),
                    }
                } else {
                    Ok(())
                }
            }
            BehaviorEval::Value(_) => Ok(()),
        };
        match state {
            Ok(_) => match &mut self.prop {
                BehaviorEval::Eval { eval: _, handle } => {
                    match eval::<T>(&handle, node, scripts, script_ctx_handles, script_ctxs) {
                        Some(Ok(val)) => {
                            self.value = Some(val.clone());
                            Some(Ok(()))
                        }
                        Some(Err(err)) => Some(Err(err)),
                        None => None,
                    }
                }
                BehaviorEval::Value(val) => {
                    self.value = Some(val.clone());
                    Some(Ok(()))
                }
            },
            Err(err) => Some(Err(err)),
        }
    }
}

fn make_handle(
    eval: impl Into<Cow<'static, str>>,
    node: &BehaviorNode,
    scripts: &mut ResMut<Assets<Script>>,
    script_ctx_handles: &Query<&Handle<ScriptContext>>,
    script_ctxs: &mut Assets<ScriptContext>,
) -> Result<Handle<Script>, String> {
    // if we have a script context handle, compile the script
    // script context are stored in the tree entity
    if let Some(script_ctx_handle) = script_ctx_handles.get(node.tree).ok() {
        // if we have a script context, compile the script
        if let Some(script_ctx) = script_ctxs.get_mut(&script_ctx_handle) {
            let mut script = Script::default();
            script.script = eval.into();
            match script.compile(script_ctx) {
                Ok(_) => {
                    let script_handle = scripts.add(script);
                    return Ok(script_handle.clone());
                }
                Err(err) => {
                    error!("{:#?}", err);
                    return Err(err.to_string());
                }
            }
        } else {
            error!("Invalid script context handle");
            return Err("Invalid script context handle".into());
        }
    } else {
        error!("Cannot find script context handle in tree entity");
        return Err("Cannot find script context handle in tree entity".into());
    }
}

/// Eval the script
fn eval<T: Reflect + Default + Clone + for<'de> Deserialize<'de>>(
    handle: &Option<Handle<Script>>,
    node: &BehaviorNode,
    scripts: &Assets<Script>,
    script_ctx_handles: &Query<&Handle<ScriptContext>>,
    script_ctxs: &mut Assets<ScriptContext>,
) -> Option<Result<T, String>> {
    if let Some(script_asset) = handle
        .as_ref()
        .and_then(|script_handle| scripts.get(&script_handle))
    {
        if let Some(script_ctx_handle) = script_ctx_handles.get(node.tree).ok() {
            if let Some(script_ctx) = script_ctxs.get_mut(&script_ctx_handle) {
                let result = script_asset.eval::<T>(script_ctx);
                match result {
                    Ok(result) => {
                        return Some(Ok(result));
                    }
                    Err(err) => {
                        error!("{:#?}", err);
                        return Some(Err(err.to_string()));
                    }
                };
            } else {
                error!("Invalid script context handle");
                return Some(Err("Invalid script context handle".into()));
            }
        } else {
            error!("Cannot find script context handle in tree entity");
            return Some(Err(
                "Cannot find script context handle in tree entity".into()
            ));
        };
    } else {
        // Still evaluating
        return None;
    }
}
