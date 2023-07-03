use crate::prelude::*;
use bevy::{prelude::*, reflect::TypeRegistry};
use serde::{Deserialize, Serialize};
use simula_script::{Script, ScriptContext};
use std::borrow::Cow;
use bevy_inspector_egui::egui;

impl BehaviorUI for BehaviorProperty<String> {

    fn ui_readonly(
        &self,
        _state: Option<protocol::BehaviorState>,
        ui: &mut bevy_inspector_egui::egui::Ui,
        _type_registry: &TypeRegistry,
    ) {
        match &self.prop {
            BehaviorEval::Value(value) => {
                ui.label(format!("{}", value));
            }
            BehaviorEval::Eval { eval, .. } => {
                ui.label(format!("= {}", eval.as_ref()));
            }
        }
    }

}








// fn ui(
//     &mut self,
//     _state: Option<protocol::BehaviorState>,
//     ui: &mut bevy_inspector_egui::egui::Ui,
//     type_registry: &TypeRegistry,
// ) -> bool {
//     let type_registry = type_registry.read();
//     bevy_inspector_egui::reflect_inspector::ui_for_value(
//         self.as_reflect_mut(),
//         ui,
//         &type_registry,
//     )
// }

// /// ui readonly inspector for behavior properties
// fn ui_readonly(
//     &self,
//     _state: Option<protocol::BehaviorState>,
//     ui: &mut bevy_inspector_egui::egui::Ui,
//     type_registry: &TypeRegistry,
// ) {
//     let type_registry = type_registry.read();
//     bevy_inspector_egui::reflect_inspector::ui_for_value_readonly(
//         self.as_reflect(),
//         ui,
//         &type_registry,
//     )
// }


// #[derive(Debug, Reflect, FromReflect, Clone, Deserialize, Serialize)]
// pub enum BehaviorEval<T: Reflect + Default> {
//     Value(T),
//     Eval {
//         eval: Cow<'static, str>,
//         #[serde(skip)]
//         #[reflect(ignore)]
//         handle: Option<Handle<Script>>,
//     },
// }

// impl<T: Reflect + Default> Default for BehaviorEval<T> {
//     fn default() -> Self {
//         Self::Value(T::default())
//     }
// }

// #[derive(Debug, Reflect, FromReflect, Clone, Deserialize, Serialize, Default)]
// pub struct BehaviorProperty<T: Reflect + Default> {
//     pub prop: BehaviorEval<T>,
//     #[serde(skip)]
//     pub value: Option<T>,
// }
