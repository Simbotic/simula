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
pub enum BehaviorPropValue<T: Reflect + Default> {
    #[default]
    None,
    Some(T),
    Err(String),
}

pub trait BehaviorProp
where
    <Self::ValueType as TryFrom<Self::ScriptType>>::Error: std::fmt::Debug,
{
    type ValueType: Reflect + Default + Clone + TryFrom<Self::ScriptType>;
    type InputType: Reflect + Default + Clone;
    type ScriptType: Reflect + Default + Clone;

    fn prop(&self) -> &BehaviorEval<Self::ValueType>;
    fn prop_mut(&mut self) -> &mut BehaviorEval<Self::ValueType>;

    fn value(&self) -> &BehaviorPropValue<Self::ValueType>;
    fn value_mut(&mut self) -> &mut BehaviorPropValue<Self::ValueType>;

    fn input(&self) -> &Option<Self::InputType>;
    fn input_mut(&mut self) -> &mut Option<Self::InputType>;

    fn fetch(
        &mut self,
        node: &BehaviorNode,
        scripts: &mut ResMut<Assets<Script>>,
        script_ctx_handles: &Query<&Handle<ScriptContext>>,
        script_ctxs: &mut Assets<ScriptContext>,
    ) -> Option<Result<(), String>> {
        let state: Result<(), String> = match self.prop_mut() {
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

        let value;
        let res = match state {
            Ok(_) => match self.prop() {
                BehaviorEval::Eval { eval: _, handle } => {
                    match eval::<Self::ValueType, Self::ScriptType>(
                        &handle,
                        node,
                        scripts,
                        script_ctx_handles,
                        script_ctxs,
                    ) {
                        Some(Ok(val)) => {
                            value = Some(BehaviorPropValue::Some(val.clone()));
                            Some(Ok(()))
                        }
                        Some(Err(err)) => {
                            value = Some(BehaviorPropValue::Err(err.clone()));
                            Some(Err(err))
                        }
                        None => {
                            value = Some(BehaviorPropValue::None);
                            None
                        }
                    }
                }
                BehaviorEval::Value(val) => {
                    value = Some(BehaviorPropValue::Some(val.clone()));
                    Some(Ok(()))
                }
            },
            Err(err) => {
                value = Some(BehaviorPropValue::Err(err.clone()));
                Some(Err(err))
            }
        };

        if let Some(value) = value {
            *self.value_mut() = value;
        }

        res
    }
}

#[derive(Debug, Reflect, FromReflect, Clone, Deserialize, Serialize, Default)]
pub struct BehaviorPropGeneric<
    ValueType: Reflect + Default + Clone + From<ScriptType>,
    InputType: Reflect + Default = ValueType,
    ScriptType: Reflect + Default = ValueType,
> {
    pub prop: BehaviorEval<ValueType>,
    #[serde(skip)]
    pub value: BehaviorPropValue<ValueType>,
    #[serde(skip)]
    #[reflect(ignore)]
    pub input: Option<InputType>,
    #[serde(skip)]
    #[reflect(ignore)]
    pub _phantom: std::marker::PhantomData<ScriptType>,
}

impl<ValueType, InputType, ScriptType> BehaviorProp
    for BehaviorPropGeneric<ValueType, InputType, ScriptType>
where
    ValueType: Reflect + Default + Clone + From<ScriptType>,
    InputType: Reflect + Default + Clone,
    ScriptType: Reflect + Default + Clone,
{
    type ValueType = ValueType;
    type InputType = InputType;
    type ScriptType = ScriptType;

    fn prop(&self) -> &BehaviorEval<Self::ValueType> {
        &self.prop
    }

    fn prop_mut(&mut self) -> &mut BehaviorEval<Self::ValueType> {
        &mut self.prop
    }

    fn value(&self) -> &BehaviorPropValue<Self::ValueType> {
        &self.value
    }

    fn value_mut(&mut self) -> &mut BehaviorPropValue<Self::ValueType> {
        &mut self.value
    }

    fn input(&self) -> &Option<Self::InputType> {
        &self.input
    }

    fn input_mut(&mut self) -> &mut Option<Self::InputType> {
        &mut self.input
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
fn eval<ValueType: Reflect + Default + Clone + TryFrom<ScriptType>, ScriptType: Reflect + Clone>(
    handle: &Option<Handle<Script>>,
    node: &BehaviorNode,
    scripts: &Assets<Script>,
    script_ctx_handles: &Query<&Handle<ScriptContext>>,
    script_ctxs: &mut Assets<ScriptContext>,
) -> Option<Result<ValueType, String>>
where
    <ValueType as TryFrom<ScriptType>>::Error: std::fmt::Debug,
{
    if let Some(script_asset) = handle
        .as_ref()
        .and_then(|script_handle| scripts.get(&script_handle))
    {
        if let Some(script_ctx_handle) = script_ctx_handles.get(node.tree).ok() {
            if let Some(script_ctx) = script_ctxs.get_mut(&script_ctx_handle) {
                let result = script_asset.eval::<ScriptType>(script_ctx);
                match result {
                    Ok(result) => {
                        let result = ValueType::try_from(result);
                        match result {
                            Ok(result) => return Some(Ok(result)),
                            Err(err) => {
                                error!("{:#?}", err);
                                let err = format!("{:#?}", err);
                                return Some(Err(err.to_string()));
                            }
                        }
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
