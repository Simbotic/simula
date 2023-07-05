use crate::prelude::*;
use bevy::{prelude::*, reflect::TypeRegistry};
use bevy_inspector_egui::egui;
use serde::{Deserialize, Serialize};
use simula_core::epath::EPath;
use std::borrow::Cow;
use std::str::FromStr;

const PROP_ICON_COLOR: egui::Color32 = egui::Color32::GRAY;
const PROP_LABEL_COLOR: egui::Color32 = egui::Color32::GRAY;
const PROP_VALUE_COLOR: egui::Color32 = egui::Color32::LIGHT_GRAY;

const PROP_NONE_COLOR: egui::Color32 = egui::Color32::LIGHT_GRAY;
const PROP_SOME_COLOR: egui::Color32 = egui::Color32::GREEN;
const PROP_ERR_COLOR: egui::Color32 = egui::Color32::RED;

const PROP_VALUE_ICON: &str = "=";
const PROP_EVAL_ICON: &str = "∑";

#[derive(Debug, Reflect, FromReflect, Clone, Deserialize, Serialize, Default)]
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
    type InputType = Cow<'static, str>;
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

    fn input(&self) -> &Option<Self::InputType> {
        &self.input
    }

    fn input_mut(&mut self) -> &mut Option<Self::InputType> {
        &mut self.input
    }
}

impl BehaviorUI for BehaviorPropStr {
    fn ui(
        &mut self,
        label: Option<&str>,
        _state: Option<protocol::BehaviorState>,
        ui: &mut bevy_inspector_egui::egui::Ui,
        _type_registry: &TypeRegistry,
    ) -> bool {
        let mut editing_text = match &self.prop {
            BehaviorEval::Value(value) => value.to_string(),
            BehaviorEval::Eval { eval, .. } => eval.to_owned().into(),
        };

        let mut changed = false;
        ui.vertical(|ui| {
            let label = label.unwrap_or("");
            let label = egui::RichText::new(format!("🔠 {}", label))
                .small()
                .color(PROP_LABEL_COLOR);
            ui.label(label);
            ui.horizontal(|ui| {
                ui.add_space(10.0);

                let icon = match &self.prop {
                    BehaviorEval::Value(_) => {
                        egui::RichText::new(PROP_VALUE_ICON).color(PROP_ICON_COLOR)
                    }
                    BehaviorEval::Eval { .. } => {
                        egui::RichText::new(PROP_EVAL_ICON).color(PROP_ICON_COLOR)
                    }
                };
                if ui.button(icon).clicked() {
                    changed |= true;
                    self.prop = match &self.prop {
                        BehaviorEval::Value(value) => BehaviorEval::Eval {
                            eval: value.to_owned().into(),
                            handle: None,
                        },
                        BehaviorEval::Eval { eval, .. } => {
                            BehaviorEval::Value(eval.to_owned().into())
                        }
                    };
                }

                changed |= match &self.prop {
                    BehaviorEval::Value(_) => ui
                        .add(egui::TextEdit::singleline(&mut editing_text).desired_width(180.0))
                        .changed(),
                    BehaviorEval::Eval { .. } => ui
                        .add(
                            egui::TextEdit::multiline(&mut editing_text)
                                .desired_width(180.0)
                                .code_editor(),
                        )
                        .changed(),
                };
            });
        });

        if changed {
            match &mut self.prop {
                BehaviorEval::Value(value) => {
                    *value = editing_text.into();
                }
                BehaviorEval::Eval { eval, .. } => {
                    *eval = editing_text.to_owned().into();
                }
            }
        }

        changed
    }

    fn ui_readonly(
        &self,
        label: Option<&str>,
        state: Option<protocol::BehaviorState>,
        ui: &mut bevy_inspector_egui::egui::Ui,
        _type_registry: &TypeRegistry,
    ) {
        ui.vertical(|ui| {
            let label = label.unwrap_or("");
            let label = egui::RichText::new(format!("🔠 {}", label))
                .small()
                .color(PROP_LABEL_COLOR);
            ui.label(label);
            ui.horizontal(|ui| {
                ui.add_space(10.0);

                let icon = match &self.prop {
                    BehaviorEval::Value(_) => egui::RichText::new(PROP_VALUE_ICON),
                    BehaviorEval::Eval { .. } => egui::RichText::new(PROP_EVAL_ICON),
                };

                if state.is_some() {
                    match &self.value {
                        BehaviorPropValue::None => {
                            ui.label(icon.color(PROP_NONE_COLOR));
                            ui.label(egui::RichText::new("...").color(PROP_VALUE_COLOR));
                        }
                        BehaviorPropValue::Some(value) => {
                            ui.label(icon.color(PROP_SOME_COLOR));
                            ui.label(
                                egui::RichText::new(value.to_string()).color(PROP_VALUE_COLOR),
                            );
                        }
                        BehaviorPropValue::Err(err) => {
                            ui.label(icon.color(PROP_ERR_COLOR));
                            ui.label(egui::RichText::new(err.to_string()).color(PROP_VALUE_COLOR));
                        }
                    }
                } else {
                    ui.label(icon.color(PROP_ICON_COLOR));
                    match &self.prop {
                        BehaviorEval::Value(value) => {
                            let mut content = value.as_ref();
                            ui.add(egui::TextEdit::singleline(&mut content).desired_width(180.0))
                                .on_hover_text(content);
                        }
                        BehaviorEval::Eval { eval, .. } => {
                            let mut content = eval.as_ref();
                            ui.add(egui::TextEdit::singleline(&mut content).desired_width(180.0))
                                .on_hover_text(content);
                        }
                    };
                }
            });
        });
    }
}

#[derive(Debug, Reflect, FromReflect, Clone, Deserialize, Serialize, Default)]
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
    type InputType = Cow<'static, str>;
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

    fn input(&self) -> &Option<Self::InputType> {
        &self.input
    }

    fn input_mut(&mut self) -> &mut Option<Self::InputType> {
        &mut self.input
    }
}

impl BehaviorUI for BehaviorPropEPath {
    fn ui(
        &mut self,
        label: Option<&str>,
        _state: Option<protocol::BehaviorState>,
        ui: &mut bevy_inspector_egui::egui::Ui,
        _type_registry: &TypeRegistry,
    ) -> bool {
        let mut editing_text = if let Some(in_progress_editing) = &self.input {
            in_progress_editing.to_string()
        } else {
            match &self.prop {
                BehaviorEval::Value(value) => value.to_string(),
                BehaviorEval::Eval { eval, .. } => eval.to_owned().into(),
            }
        };

        let mut changed = false;
        let mut commit = false;
        ui.vertical(|ui| {
            let label = label.unwrap_or("");
            let label = egui::RichText::new(format!("🌎 {}", label))
                .small()
                .color(PROP_LABEL_COLOR);
            ui.label(label);
            ui.horizontal(|ui| {
                ui.add_space(10.0);

                let icon = match &self.prop {
                    BehaviorEval::Value(_) => {
                        egui::RichText::new(PROP_VALUE_ICON).color(PROP_ICON_COLOR)
                    }
                    BehaviorEval::Eval { .. } => {
                        egui::RichText::new(PROP_EVAL_ICON).color(PROP_ICON_COLOR)
                    }
                };
                if ui.button(icon).clicked() {
                    changed |= true;
                    self.prop = match &self.prop {
                        BehaviorEval::Value(value) => BehaviorEval::Eval {
                            eval: value.to_string().into(),
                            handle: None,
                        },
                        BehaviorEval::Eval { .. } => BehaviorEval::Value(EPath::default()),
                    };
                }

                changed |= match &self.prop {
                    BehaviorEval::Value(_) => {
                        let epath = EPath::from_str(&editing_text);
                        let frame_color = if let Err(_) = epath {
                            egui::Color32::RED
                        } else {
                            egui::Color32::TRANSPARENT
                        };
                        let frame = egui::Frame::none()
                            .fill(frame_color)
                            .inner_margin(3.0)
                            .rounding(4.0);
                        let res = frame
                            .show(ui, |ui| {
                                ui.add(
                                    egui::TextEdit::singleline(&mut editing_text)
                                        .desired_width(180.0)
                                        .cursor_at_end(true),
                                )
                            })
                            .inner;
                        if res.lost_focus() {
                            commit = true;
                        }
                        res.changed()
                    }
                    BehaviorEval::Eval { .. } => {
                        let res = ui.add(
                            egui::TextEdit::multiline(&mut editing_text)
                                .desired_width(180.0)
                                .code_editor(),
                        );
                        res.changed()
                    }
                };
            });
        });

        if commit {
            self.input = None;
            changed = true;
            match &mut self.prop {
                BehaviorEval::Value(value) => {
                    let epath = EPath::from_str(&editing_text);
                    if let Ok(epath) = epath {
                        println!("epath: {:?}", epath);
                        *value = epath;
                    }
                }
                BehaviorEval::Eval { eval, .. } => {
                    *eval = editing_text.to_owned().into();
                }
            }
        } else if changed {
            self.input = Some(editing_text.into());
        }

        changed
    }

    fn ui_readonly(
        &self,
        label: Option<&str>,
        state: Option<protocol::BehaviorState>,
        ui: &mut bevy_inspector_egui::egui::Ui,
        _type_registry: &TypeRegistry,
    ) {
        ui.vertical(|ui| {
            let label = label.unwrap_or("");
            let label = egui::RichText::new(format!("🌎 {}", label))
                .small()
                .color(PROP_LABEL_COLOR);
            ui.label(label);
            ui.horizontal(|ui| {
                ui.add_space(10.0);

                let icon = match &self.prop {
                    BehaviorEval::Value(_) => egui::RichText::new(PROP_VALUE_ICON),
                    BehaviorEval::Eval { .. } => egui::RichText::new(PROP_EVAL_ICON),
                };

                if state.is_some() {
                    match &self.value {
                        BehaviorPropValue::None => {
                            ui.label(icon.color(PROP_NONE_COLOR));
                            ui.label(egui::RichText::new("...").color(PROP_VALUE_COLOR));
                        }
                        BehaviorPropValue::Some(value) => {
                            ui.label(icon.color(PROP_SOME_COLOR));
                            ui.label(
                                egui::RichText::new(value.to_string()).color(PROP_VALUE_COLOR),
                            )
                            .on_disabled_hover_text(value.to_string());
                        }
                        BehaviorPropValue::Err(err) => {
                            ui.label(icon.color(PROP_ERR_COLOR));
                            ui.label(egui::RichText::new(err.to_string()).color(PROP_VALUE_COLOR));
                        }
                    }
                } else {
                    ui.label(icon.color(PROP_ICON_COLOR));
                    match &self.prop {
                        BehaviorEval::Value(value) => {
                            let content = value.to_string();
                            let mut content = content.as_str();
                            ui.add(egui::TextEdit::singleline(&mut content).desired_width(180.0))
                                .on_hover_text(content);
                        }
                        BehaviorEval::Eval { eval, .. } => {
                            let mut content = eval.as_ref();
                            ui.add(egui::TextEdit::singleline(&mut content).desired_width(180.0))
                                .on_hover_text(content);
                        }
                    };
                }
            });
        });
    }
}
