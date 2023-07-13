use crate::prelude::*;
use bevy::{prelude::*, reflect::TypeRegistry};
use bevy_inspector_egui::{egui, reflect_inspector};
use simula_core::epath::EPath;
use std::str::FromStr;

const PROP_ICON_COLOR: egui::Color32 = egui::Color32::GRAY;
const PROP_LABEL_COLOR: egui::Color32 = egui::Color32::GRAY;
const PROP_VALUE_COLOR: egui::Color32 = egui::Color32::LIGHT_GRAY;

const PROP_NONE_COLOR: egui::Color32 = egui::Color32::LIGHT_GRAY;
const PROP_SOME_COLOR: egui::Color32 = egui::Color32::GREEN;
const PROP_ERR_COLOR: egui::Color32 = egui::Color32::RED;

const PROP_TEXT_WIDTH: f32 = 180.0;

const PROP_FRAME_COLOR: egui::Color32 = egui::Color32::from_rgb(32, 30, 35);
const PROP_FRAME_RADIUS: f32 = 2.0;
const PROP_FRAME_WIDTH: f32 = 200.0;
const PROP_FRAME_INNER_MARGIN: egui::Margin = egui::Margin {
    top: 2.0,
    bottom: 2.0,
    left: 2.0,
    right: 0.0,
};
const PROP_FRAME_OUTER_MARGIN: egui::Margin = egui::Margin {
    top: 0.0,
    bottom: 0.0,
    left: -5.0,
    right: -5.0,
};

const PROP_VALUE_ICON: &str = "=";
const PROP_EVAL_ICON: &str = "λ";

impl<ValueType, ScriptType> BehaviorUI for BehaviorPropGeneric<ValueType, ScriptType>
where
    ValueType: FromReflect + Reflect + Default + Clone + From<ScriptType>,
    ScriptType: Reflect + Default,
{
    fn ui(
        &mut self,
        label: Option<&str>,
        _state: Option<protocol::BehaviorState>,
        ui: &mut bevy_inspector_egui::egui::Ui,
        type_registry: &TypeRegistry,
    ) -> bool {
        egui::Frame::none()
            .inner_margin(PROP_FRAME_INNER_MARGIN)
            .outer_margin(PROP_FRAME_OUTER_MARGIN)
            .rounding(PROP_FRAME_RADIUS)
            .fill(PROP_FRAME_COLOR)
            .show(ui, |ui| {
                ui.set_width(PROP_FRAME_WIDTH);

                let mut editing_text = match &self.prop {
                    BehaviorEval::Value(_) => "".to_string(),
                    BehaviorEval::Eval { eval, .. } => eval.to_owned().into(),
                };

                let mut changed = false;
                ui.vertical(|ui| {
                    let label = label.unwrap_or("");
                    let label = egui::RichText::new(format!("{}", label))
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
                                BehaviorEval::Value(_) => BehaviorEval::Eval {
                                    eval: "".into(),
                                    handle: None,
                                },
                                BehaviorEval::Eval { .. } => {
                                    BehaviorEval::Value(ValueType::default())
                                }
                            };
                        }

                        changed |= match &mut self.prop {
                            BehaviorEval::Value(value) => {
                                let type_registry = type_registry.read();
                                reflect_inspector::ui_for_value(
                                    value.as_reflect_mut(),
                                    ui,
                                    &type_registry,
                                )
                            }
                            BehaviorEval::Eval { .. } => ui
                                .add(
                                    egui::TextEdit::multiline(&mut editing_text)
                                        .desired_width(PROP_TEXT_WIDTH)
                                        .code_editor(),
                                )
                                .changed(),
                        };
                    });
                });

                if changed {
                    match &mut self.prop {
                        BehaviorEval::Value(_) => {
                            // handled by reflect_inspector
                        }
                        BehaviorEval::Eval { eval, .. } => {
                            *eval = editing_text.to_owned().into();
                        }
                    }
                }

                changed
            })
            .inner
    }

    fn ui_readonly(
        &self,
        label: Option<&str>,
        state: Option<protocol::BehaviorState>,
        ui: &mut bevy_inspector_egui::egui::Ui,
        type_registry: &TypeRegistry,
    ) {
        egui::Frame::none()
            .inner_margin(PROP_FRAME_INNER_MARGIN)
            .outer_margin(PROP_FRAME_OUTER_MARGIN)
            .rounding(PROP_FRAME_RADIUS)
            .fill(PROP_FRAME_COLOR)
            .show(ui, |ui| {
                ui.set_width(PROP_FRAME_WIDTH);

                ui.vertical(|ui| {
                    let label = label.unwrap_or("");
                    let label = egui::RichText::new(format!("{}", label))
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
                                    let type_registry = type_registry.read();
                                    reflect_inspector::ui_for_value_readonly(
                                        value.as_reflect(),
                                        ui,
                                        &type_registry,
                                    );
                                }
                                BehaviorPropValue::Err(err) => {
                                    ui.label(icon.color(PROP_ERR_COLOR));
                                    ui.label(
                                        egui::RichText::new(err.to_string())
                                            .color(PROP_VALUE_COLOR),
                                    );
                                }
                            }
                        } else {
                            ui.label(icon.color(PROP_ICON_COLOR));
                            match &self.prop {
                                BehaviorEval::Value(value) => {
                                    let type_registry = type_registry.read();
                                    reflect_inspector::ui_for_value_readonly(
                                        value.as_reflect(),
                                        ui,
                                        &type_registry,
                                    );
                                }
                                BehaviorEval::Eval { eval, .. } => {
                                    let mut content = eval.as_ref();
                                    ui.add(
                                        egui::TextEdit::singleline(&mut content)
                                            .desired_width(PROP_TEXT_WIDTH),
                                    )
                                    .on_hover_text(content);
                                }
                            };
                        }
                    });
                });
            });
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
        egui::Frame::none()
            .inner_margin(PROP_FRAME_INNER_MARGIN)
            .outer_margin(PROP_FRAME_OUTER_MARGIN)
            .rounding(PROP_FRAME_RADIUS)
            .fill(PROP_FRAME_COLOR)
            .show(ui, |ui| {
                ui.set_width(PROP_FRAME_WIDTH);

                let mut editing_text = match &self.prop {
                    BehaviorEval::Value(value) => value.to_string(),
                    BehaviorEval::Eval { eval, .. } => eval.to_owned().into(),
                };

                let mut changed = false;
                ui.vertical(|ui| {
                    let label = label.unwrap_or("");
                    let label = egui::RichText::new(format!("{}", label))
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
                                .add(
                                    egui::TextEdit::singleline(&mut editing_text)
                                        .desired_width(PROP_TEXT_WIDTH),
                                )
                                .changed(),
                            BehaviorEval::Eval { .. } => ui
                                .add(
                                    egui::TextEdit::multiline(&mut editing_text)
                                        .desired_width(PROP_TEXT_WIDTH)
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
            })
            .inner
    }

    fn ui_readonly(
        &self,
        label: Option<&str>,
        state: Option<protocol::BehaviorState>,
        ui: &mut bevy_inspector_egui::egui::Ui,
        _type_registry: &TypeRegistry,
    ) {
        egui::Frame::none()
            .inner_margin(PROP_FRAME_INNER_MARGIN)
            .outer_margin(PROP_FRAME_OUTER_MARGIN)
            .rounding(PROP_FRAME_RADIUS)
            .fill(PROP_FRAME_COLOR)
            .show(ui, |ui| {
                ui.set_width(PROP_FRAME_WIDTH);

                ui.vertical(|ui| {
                    let label = label.unwrap_or("");
                    let label = egui::RichText::new(format!("{}", label))
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
                                    ui.add(
                                        egui::TextEdit::singleline(&mut value.as_ref())
                                            .desired_width(PROP_TEXT_WIDTH)
                                            .text_color(PROP_VALUE_COLOR),
                                    )
                                    .on_hover_text(value.as_ref());
                                }
                                BehaviorPropValue::Err(err) => {
                                    ui.label(icon.color(PROP_ERR_COLOR));
                                    ui.label(
                                        egui::RichText::new(err.to_string())
                                            .color(PROP_VALUE_COLOR),
                                    );
                                }
                            }
                        } else {
                            ui.label(icon.color(PROP_ICON_COLOR));
                            match &self.prop {
                                BehaviorEval::Value(value) => {
                                    let mut content = value.as_ref();
                                    ui.add(
                                        egui::TextEdit::singleline(&mut content)
                                            .desired_width(PROP_TEXT_WIDTH),
                                    )
                                    .on_hover_text(content);
                                }
                                BehaviorEval::Eval { eval, .. } => {
                                    let mut content = eval.as_ref();
                                    ui.add(
                                        egui::TextEdit::singleline(&mut content)
                                            .desired_width(PROP_TEXT_WIDTH),
                                    )
                                    .on_hover_text(content);
                                }
                            };
                        }
                    });
                });
            });
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
        egui::Frame::none()
            .inner_margin(PROP_FRAME_INNER_MARGIN)
            .outer_margin(PROP_FRAME_OUTER_MARGIN)
            .rounding(PROP_FRAME_RADIUS)
            .fill(PROP_FRAME_COLOR)
            .show(ui, |ui| {
                ui.set_width(PROP_FRAME_WIDTH);

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
                    let label = egui::RichText::new(format!("{}", label))
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
                                                .desired_width(PROP_TEXT_WIDTH)
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
                                        .desired_width(PROP_TEXT_WIDTH)
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
            })
            .inner
    }

    fn ui_readonly(
        &self,
        label: Option<&str>,
        state: Option<protocol::BehaviorState>,
        ui: &mut bevy_inspector_egui::egui::Ui,
        _type_registry: &TypeRegistry,
    ) {
        egui::Frame::none()
            .inner_margin(PROP_FRAME_INNER_MARGIN)
            .outer_margin(PROP_FRAME_OUTER_MARGIN)
            .rounding(PROP_FRAME_RADIUS)
            .fill(PROP_FRAME_COLOR)
            .show(ui, |ui| {
                ui.set_width(PROP_FRAME_WIDTH);

                ui.vertical(|ui| {
                    let label = label.unwrap_or("");
                    let label = egui::RichText::new(format!("{}", label))
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
                                    let value = value.to_string();
                                    ui.add(
                                        egui::TextEdit::singleline(&mut value.as_str())
                                            .desired_width(PROP_TEXT_WIDTH),
                                    )
                                    .on_hover_text(value);
                                }
                                BehaviorPropValue::Err(err) => {
                                    ui.label(icon.color(PROP_ERR_COLOR));
                                    ui.label(
                                        egui::RichText::new(err.to_string())
                                            .color(PROP_VALUE_COLOR),
                                    );
                                }
                            }
                        } else {
                            ui.label(icon.color(PROP_ICON_COLOR));
                            match &self.prop {
                                BehaviorEval::Value(value) => {
                                    let content = value.to_string();
                                    let mut content = content.as_str();
                                    ui.add(
                                        egui::TextEdit::singleline(&mut content)
                                            .desired_width(PROP_TEXT_WIDTH),
                                    )
                                    .on_hover_text(content);
                                }
                                BehaviorEval::Eval { eval, .. } => {
                                    let mut content = eval.as_ref();
                                    ui.add(
                                        egui::TextEdit::singleline(&mut content)
                                            .desired_width(PROP_TEXT_WIDTH),
                                    )
                                    .on_hover_text(content);
                                }
                            };
                        }
                    });
                });
            });
    }
}

impl<Prop> BehaviorUI for BehaviorPropOption<Prop>
where
    Prop: BehaviorProp + BehaviorUI,
    <<Prop as BehaviorProp>::ValueType as TryFrom<<Prop as BehaviorProp>::ScriptType>>::Error:
        std::fmt::Debug,
{
    fn ui(
        &mut self,
        label: Option<&str>,
        state: Option<protocol::BehaviorState>,
        ui: &mut bevy_inspector_egui::egui::Ui,
        type_registry: &TypeRegistry,
    ) -> bool {
        let mut changed = false;
        let mut reset = false;

        if let Some(prop) = self.as_mut() {
            let wrapper = egui::Frame::none()
                .show(ui, |ui| changed |= prop.ui(label, state, ui, type_registry));
            if egui::Frame::none()
                .inner_margin(egui::Margin {
                    top: -2.0,
                    bottom: 0.0,
                    left: 0.0,
                    right: 0.0,
                })
                .outer_margin(egui::Margin {
                    top: -wrapper.response.rect.height(),
                    bottom: 0.0,
                    left: 0.0,
                    right: 0.0,
                })
                .fill(egui::Color32::TRANSPARENT)
                .show(ui, |ui| {
                    ui.horizontal(|ui| {
                        ui.add_space(170.0);
                        ui.small_button("-")
                    })
                    .inner
                })
                .inner
                .clicked()
            {
                reset = true;
                changed |= true;
            }
        } else {
            egui::Frame::none()
                .inner_margin(PROP_FRAME_INNER_MARGIN)
                .outer_margin(PROP_FRAME_OUTER_MARGIN)
                .rounding(PROP_FRAME_RADIUS)
                .fill(PROP_FRAME_COLOR)
                .show(ui, |ui| {
                    ui.set_width(PROP_FRAME_WIDTH);
                    ui.vertical(|ui| {
                        let label = label.unwrap_or("");
                        let label = egui::RichText::new(format!("{}", label))
                            .small()
                            .color(PROP_LABEL_COLOR);
                        ui.label(label);
                        ui.horizontal(|ui| {
                            ui.add_space(10.0);

                            if ui.button("?").clicked() {
                                **self = Some(Prop::default());
                                changed |= true;
                            }
                        });
                    });
                })
                .inner
        }

        if reset {
            **self = None;
        }
        changed
    }

    fn ui_readonly(
        &self,
        label: Option<&str>,
        state: Option<protocol::BehaviorState>,
        ui: &mut bevy_inspector_egui::egui::Ui,
        type_registry: &TypeRegistry,
    ) {
        if let Some(prop) = self.as_ref() {
            prop.ui_readonly(label, state, ui, type_registry);
        } else {
            egui::Frame::none()
                .inner_margin(PROP_FRAME_INNER_MARGIN)
                .outer_margin(PROP_FRAME_OUTER_MARGIN)
                .rounding(PROP_FRAME_RADIUS)
                .fill(PROP_FRAME_COLOR)
                .show(ui, |ui| {
                    ui.set_width(PROP_FRAME_WIDTH);

                    ui.vertical(|ui| {
                        let label = label.unwrap_or("");
                        let label = egui::RichText::new(format!("{}", label))
                            .small()
                            .color(PROP_LABEL_COLOR);
                        ui.label(label);
                        ui.horizontal(|ui| {
                            ui.add_space(10.0);
                        });
                    });
                });
        }
    }
}
