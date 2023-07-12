use crate::debug::*;
use crate::prelude::*;
use crate::ScriptContext;
use bevy::prelude::*;
use bevy_inspector_egui::prelude::*;
use serde::{Deserialize, Serialize};
use simula_script::Script;

/// A wait will succeed after a specified amount of time.
#[derive(
    Debug, Default, Component, Reflect, FromReflect, Clone, Deserialize, Serialize, InspectorOptions,
)]
#[reflect(InspectorOptions)]
pub struct Wait {
    #[serde(default)]
    #[inspector(min = 0.0, max = f64::MAX)]
    pub duration: f64,
    #[serde(default)]
    pub fail: BehaviorPropGeneric<bool>,
    #[serde(skip)]
    pub start: f64,
    #[serde(skip)]
    pub ticks: u64,
}

impl BehaviorSpec for Wait {
    const TYPE: BehaviorType = BehaviorType::Action;
    const NAME: &'static str = "Wait";
    const ICON: &'static str = "⌛";
    const DESC: &'static str = "Wait for a specified amount of time and then complete with \
    success or failure.";
}

impl BehaviorUI for Wait {
    fn ui(
        &mut self,
        _label: Option<&str>,
        state: Option<protocol::BehaviorState>,
        ui: &mut bevy_inspector_egui::egui::Ui,
        type_registry: &bevy::reflect::TypeRegistry,
    ) -> bool {
        let mut changed = false;
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
        behavior_ui_readonly!(self, fail, state, ui, type_registry);
        add_parameter_readonly(ui, type_registry, "duration", self.duration.as_reflect());
        add_parameter_readonly(ui, type_registry, "start", self.start.as_reflect());
        add_parameter_readonly(ui, type_registry, "ticks", self.ticks.as_reflect());
    }
}

pub fn run(
    time: Res<Time>,
    mut commands: Commands,
    mut waits: Query<
        (Entity, &mut Wait, &BehaviorNode, Option<&BehaviorStarted>),
        BehaviorRunQuery,
    >,
    mut scripts: ResMut<Assets<Script>>,
    script_ctx_handles: Query<&Handle<ScriptContext>>,
    mut script_ctxs: ResMut<Assets<ScriptContext>>,
) {
    for (entity, mut wait, node, started) in &mut waits {
        if let BehaviorPropValue::None = wait.fail.value {
            let result = wait
                .fail
                .fetch(node, &mut scripts, &script_ctx_handles, &mut script_ctxs);
            if let Some(Err(err)) = result {
                error!("Script errored: {:?}", err);
                commands.entity(entity).insert(BehaviorFailure);
                continue;
            }
        }

        if let BehaviorPropValue::Some(wait_fail) = &wait.fail.value.clone() {
            let elapsed = time.elapsed_seconds_f64();
            wait.ticks += 1;
            if started.is_some() {
                wait.start = elapsed;
            }
            if elapsed - wait.start > wait.duration - f64::EPSILON {
                if *wait_fail {
                    commands.entity(entity).insert(BehaviorFailure);
                } else {
                    commands.entity(entity).insert(BehaviorSuccess);
                }
            }
        }
    }
}
