use crate::prelude::*;
use bevy::prelude::*;
use serde::{Deserialize, Serialize};
use simula_script::{Script, ScriptContext};
use std::borrow::Cow;

pub fn create_script_context() -> ScriptContext {
    let mut scope = ScriptContext::new();
    let mut blackboard = simula_script::script::Map::new();
    blackboard.insert("state".into(), 0.into());
    scope.scope.push("blackboard", blackboard);
    scope
}

#[derive(Debug, Reflect, FromReflect, Clone, Deserialize, Serialize)]
pub struct BehaviorEval<T: Reflect + Default> {
    pub eval: Cow<'static, str>,
    #[serde(skip)]
    pub result: Option<T>,
    #[serde(skip)]
    #[reflect(ignore)]
    pub script_handle: Option<Handle<Script>>,
}

impl<T> BehaviorEval<T>
where
    T: Reflect + Default + Clone + for<'de> Deserialize<'de>,
{
    pub fn new(eval: impl Into<Cow<'static, str>>) -> Self {
        Self {
            eval: eval.into(),
            result: None,
            script_handle: None,
        }
    }

    /// Return a script handle, compiling the script if necessary
    pub fn make_handle(
        &mut self,
        node: &BehaviorNode,
        scripts: &mut ResMut<Assets<Script>>,
        script_ctx_handles: &Query<&Handle<ScriptContext>>,
        script_ctxs: &mut Assets<ScriptContext>,
    ) -> Result<Handle<Script>, String> {
        // if we have a handle already, return it
        if let Some(script_handle) = &self.script_handle {
            return Ok(script_handle.clone());
        }

        // if we have a script context handle, compile the script
        // script context are stored in the tree entity
        if let Some(script_ctx_handle) = script_ctx_handles.get(node.tree).ok() {
            // if we have a script context, compile the script
            if let Some(script_ctx) = script_ctxs.get_mut(&script_ctx_handle) {
                let mut script = Script::default();
                script.script = self.eval.to_owned();
                match script.compile(script_ctx) {
                    Ok(_) => {
                        let script_handle = scripts.add(script);
                        self.script_handle = Some(script_handle.clone());
                        return Ok(script_handle);
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
    pub fn eval(
        &mut self,
        node: &BehaviorNode,
        scripts: &Assets<Script>,
        script_ctx_handles: &Query<&Handle<ScriptContext>>,
        script_ctxs: &mut Assets<ScriptContext>,
    ) -> Result<(), String> {
        if let Some(script_asset) = self
            .script_handle
            .as_ref()
            .and_then(|script_handle| scripts.get(&script_handle))
        {
            self.result = None;
            if let Some(script_ctx_handle) = script_ctx_handles.get(node.tree).ok() {
                if let Some(script_ctx) = script_ctxs.get_mut(&script_ctx_handle) {
                    let result = script_asset.eval::<T>(script_ctx);
                    match result {
                        Ok(result) => {
                            self.result = Some(result.clone());
                            return Ok(());
                        }
                        Err(err) => {
                            error!("{:#?}", err);
                            return Err(err.to_string());
                        }
                    };
                } else {
                    error!("Invalid script context handle");
                    return Err("Invalid script context handle".into());
                }
            } else {
                error!("Cannot find script context handle in tree entity");
                return Err("Cannot find script context handle in tree entity".into());
            };
        } else {
            // Still evaluating
            return Ok(());
        }
    }
}

// mut scripts: ResMut<Assets<Script>>
