use crate::prelude::*;
use bevy::{ecs::system::SystemParam, prelude::*};
use serde::{Deserialize, Serialize};
use simula_core::epath::EPath;
use simula_script::{Script, ScriptContext};
use std::borrow::Cow;

#[derive(Debug, Reflect, Clone, Deserialize, Serialize)]
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

#[derive(Debug, Reflect, Clone, Deserialize, Serialize, Default)]
pub enum BehaviorPropValue<T: Reflect + Default> {
    #[default]
    None,
    Some(T),
    Err(String),
}

pub trait BehaviorProp
where
    Self: FromReflect + Reflect + Default + Clone,
    <Self::ValueType as TryFrom<Self::ScriptType>>::Error: std::fmt::Debug,
{
    type ValueType: Reflect + Default + Clone + TryFrom<Self::ScriptType>;
    type ScriptType: Reflect + Default + Clone;

    fn prop(&self) -> &BehaviorEval<Self::ValueType>;
    fn prop_mut(&mut self) -> &mut BehaviorEval<Self::ValueType>;

    fn value(&self) -> &BehaviorPropValue<Self::ValueType>;
    fn value_mut(&mut self) -> &mut BehaviorPropValue<Self::ValueType>;

    fn fetch(
        &mut self,
        node: &BehaviorNode,
        scripts: &mut ScriptQueries,
    ) -> Option<Result<(), String>> {
        let state: Result<(), String> = match self.prop_mut() {
            BehaviorEval::Eval { eval, handle } => {
                if handle.is_none() {
                    match make_handle(eval.to_owned(), node, scripts) {
                        Ok(new_handle) => {
                            *handle = Some(new_handle);
                            Ok(())
                        }
                        Err(err) => Err(err),
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
                    match eval::<Self::ValueType, Self::ScriptType>(&handle, node, scripts) {
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

#[derive(Debug, Reflect, Clone, Deserialize, Serialize, Default, Deref, DerefMut)]
pub struct BehaviorPropOption<Prop>(Option<Prop>)
where
    Prop: BehaviorProp + FromReflect + Reflect + Default + Clone + TypePath,
    <<Prop as BehaviorProp>::ValueType as TryFrom<<Prop as BehaviorProp>::ScriptType>>::Error:
        std::fmt::Debug;

impl<Prop> BehaviorPropOption<Prop>
where
    Prop: BehaviorProp + FromReflect + Reflect + Default + Clone + TypePath,
    <<Prop as BehaviorProp>::ValueType as TryFrom<<Prop as BehaviorProp>::ScriptType>>::Error:
        std::fmt::Debug,
{
}

#[derive(Debug, Reflect, Clone, Deserialize, Serialize, Default)]
pub struct BehaviorPropGeneric<
    ValueType: Reflect + FromReflect + TypePath + Default + Clone + From<ScriptType>,
    ScriptType: Reflect + TypePath + Default = ValueType,
> {
    pub prop: BehaviorEval<ValueType>,
    #[serde(skip)]
    pub value: BehaviorPropValue<ValueType>,
    #[serde(skip)]
    #[reflect(ignore)]
    pub _phantom: std::marker::PhantomData<ScriptType>,
}

impl<ValueType, ScriptType> BehaviorProp for BehaviorPropGeneric<ValueType, ScriptType>
where
    ValueType: FromReflect + Reflect + TypePath + Default + Clone + From<ScriptType>,
    ScriptType: Reflect + TypePath + Default + Clone,
{
    type ValueType = ValueType;
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
}

#[derive(Debug, Reflect, Clone, Deserialize, Serialize, Default)]
pub struct BehaviorPropStr {
    pub prop: BehaviorEval<Cow<'static, str>>,
    #[serde(skip)]
    pub value: BehaviorPropValue<Cow<'static, str>>,
    #[serde(skip)]
    #[reflect(ignore)]
    pub input: Option<Cow<'static, str>>,
}

impl BehaviorProp for BehaviorPropStr {
    type ValueType = Cow<'static, str>;
    type ScriptType = String;

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
}

#[derive(Debug, Reflect, Clone, Deserialize, Serialize, Default)]
pub struct BehaviorPropEPath {
    pub prop: BehaviorEval<EPath>,
    #[serde(skip)]
    pub value: BehaviorPropValue<EPath>,
    #[serde(skip)]
    #[reflect(ignore)]
    pub input: Option<Cow<'static, str>>,
}

impl BehaviorProp for BehaviorPropEPath {
    type ValueType = EPath;
    type ScriptType = String;

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
}

#[derive(SystemParam)]
pub struct ScriptQueries<'w, 's> {
    assets: ResMut<'w, Assets<Script>>,
    ctx_handles: Query<'w, 's, &'static Handle<ScriptContext>>,
    ctxs: ResMut<'w, Assets<ScriptContext>>,
}

fn make_handle(
    eval: impl Into<Cow<'static, str>>,
    node: &BehaviorNode,
    scripts: &mut ScriptQueries,
) -> Result<Handle<Script>, String> {
    // if we have a script context handle, compile the script
    // script context are stored in the tree entity
    if let Some(script_ctx_handle) = scripts.ctx_handles.get(node.tree).ok() {
        // if we have a script context, compile the script
        if let Some(script_ctx) = scripts.ctxs.get_mut(script_ctx_handle) {
            let mut script = Script::default();
            script.script = eval.into();
            match script.compile(script_ctx) {
                Ok(_) => {
                    let script_handle = scripts.assets.add(script);
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
    scripts: &mut ScriptQueries,
) -> Option<Result<ValueType, String>>
where
    <ValueType as TryFrom<ScriptType>>::Error: std::fmt::Debug,
{
    if let Some(script_asset) = handle
        .as_ref()
        .and_then(|script_handle| scripts.assets.get(script_handle))
    {
        if let Some(script_ctx_handle) = scripts.ctx_handles.get(node.tree).ok() {
            if let Some(script_ctx) = scripts.ctxs.get_mut(script_ctx_handle) {
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
