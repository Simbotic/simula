use crate::prelude::*;
use crate::ScriptContext;
use bevy::prelude::*;
use bevy_inspector_egui::prelude::*;
use serde::{Deserialize, Serialize};
use simula_script::Script;

#[derive(
    Debug, Default, Component, Reflect, FromReflect, Clone, Deserialize, Serialize, InspectorOptions,
)]
#[reflect(InspectorOptions)]
pub struct Debug {
    #[serde(default)]
    pub message: BehaviorPropStr,
    #[serde(default)]
    pub fail: BehaviorPropGeneric<bool>,
    #[serde(default)]
    #[inspector(min = 0.0, max = f64::MAX)]
    pub duration: f64,
    #[serde(skip)]
    pub start: f64,
    #[serde(skip)]
    pub ticks: u64,
}

impl BehaviorSpec for Debug {
    const TYPE: BehaviorType = BehaviorType::Action;
    const NAME: &'static str = "Debug";
    const ICON: &'static str = "👁";
    const DESC: &'static str = "Display a debug message and complete with success or failure";
}

pub fn add_parameter(
    ui: &mut bevy_inspector_egui::egui::Ui,
    type_registry: &bevy::reflect::TypeRegistry,
    label: &str,
    value: &mut dyn Reflect,
) -> bool {
    let mut result = false;
    let type_registry = type_registry.read();
    ui.horizontal(|ui| {
        ui.label(label);
        result = bevy_inspector_egui::reflect_inspector::ui_for_value(value, ui, &type_registry);
    });
    result
}

pub fn add_parameter_readonly(
    ui: &mut bevy_inspector_egui::egui::Ui,
    type_registry: &bevy::reflect::TypeRegistry,
    label: &str,
    value: &dyn Reflect,
) {
    let type_registry = type_registry.read();
    ui.horizontal(|ui| {
        ui.label(label);
        bevy_inspector_egui::reflect_inspector::ui_for_value_readonly(value, ui, &type_registry);
    });
}

impl BehaviorUI for Debug {
    fn ui(
        &mut self,
        _label: Option<&str>,
        state: Option<protocol::BehaviorState>,
        ui: &mut bevy_inspector_egui::egui::Ui,
        type_registry: &bevy::reflect::TypeRegistry,
    ) -> bool {
        let mut changed = false;
        changed |= behavior_ui!(self, message, state, ui, type_registry);
        changed |= behavior_ui!(self, fail, state, ui, type_registry);
        changed |= add_parameter(
            ui,
            type_registry,
            "duration",
            self.duration.as_reflect_mut(),
        );
        changed |= add_parameter(ui, type_registry, "start", self.start.as_reflect_mut());
        changed |= add_parameter(ui, type_registry, "ticks", self.ticks.as_reflect_mut());
        changed
    }

    fn ui_readonly(
        &self,
        _label: Option<&str>,
        state: Option<protocol::BehaviorState>,
        ui: &mut bevy_inspector_egui::egui::Ui,
        type_registry: &bevy::reflect::TypeRegistry,
    ) {
        behavior_ui_readonly!(self, message, state, ui, type_registry);
        behavior_ui_readonly!(self, fail, state, ui, type_registry);
        add_parameter_readonly(ui, type_registry, "duration", self.duration.as_reflect());
        add_parameter_readonly(ui, type_registry, "start", self.start.as_reflect());
        add_parameter_readonly(ui, type_registry, "ticks", self.ticks.as_reflect());
    }
}

pub fn run(
    time: Res<Time>,
    mut commands: Commands,
    mut debug_actions: Query<
        (
            Entity,
            &mut Debug,
            Option<&Name>,
            &BehaviorNode,
            Option<&BehaviorStarted>,
        ),
        BehaviorRunQuery,
    >,
    mut scripts: ResMut<Assets<Script>>,
    script_ctx_handles: Query<&Handle<ScriptContext>>,
    mut script_ctxs: ResMut<Assets<ScriptContext>>,
) {
    for (entity, mut debug_action, name, node, started) in &mut debug_actions {
        if let BehaviorPropValue::None = debug_action.message.value {
            let result = debug_action.message.fetch(
                node,
                &mut scripts,
                &script_ctx_handles,
                &mut script_ctxs,
            );
            if let Some(Err(err)) = result {
                error!("Script errored: {:?}", err);
                commands.entity(entity).insert(BehaviorFailure);
                continue;
            }
        }

        if let BehaviorPropValue::None = debug_action.fail.value {
            let result =
                debug_action
                    .fail
                    .fetch(node, &mut scripts, &script_ctx_handles, &mut script_ctxs);
            if let Some(Err(err)) = result {
                error!("Script errored: {:?}", err);
                commands.entity(entity).insert(BehaviorFailure);
                continue;
            }
        }

        if let (
            BehaviorPropValue::Some(debug_action_message),
            BehaviorPropValue::Some(debug_action_fail),
        ) = (
            &debug_action.message.value.clone(),
            &debug_action.fail.value.clone(),
        ) {
            let elapsed = time.elapsed_seconds_f64();
            debug_action.ticks += 1;
            if started.is_some() {
                debug_action.start = elapsed;
                let name = name.map(|name| name.as_str()).unwrap_or("");
                info!("[{}:{}] {}", entity.index(), name, debug_action_message);
            }
            if elapsed - debug_action.start > debug_action.duration - f64::EPSILON {
                if *debug_action_fail {
                    commands.entity(entity).insert(BehaviorFailure);
                } else {
                    commands.entity(entity).insert(BehaviorSuccess);
                }
            }
        }
    }
}
