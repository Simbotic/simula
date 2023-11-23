use crate::prelude::*;
use crate::property_ui_readonly;
use bevy::prelude::*;
use bevy_inspector_egui::prelude::*;
use serde::{Deserialize, Serialize};

/// A wait will succeed after a specified amount of time.
#[derive(
    Debug, Default, Component, Reflect, Clone, Deserialize, Serialize, InspectorOptions,
)]
#[reflect(InspectorOptions)]
pub struct Wait {
    #[serde(default)]
    pub duration: BehaviorPropGeneric<f64>,
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
        type_registry: &bevy::reflect::TypeRegistryArc,
    ) -> bool {
        let mut changed = false;
        changed |= behavior_ui!(self, fail, state, ui, type_registry);
        changed |= behavior_ui!(self, duration, state, ui, type_registry);
        changed
    }

    fn ui_readonly(
        &self,
        _label: Option<&str>,
        state: Option<protocol::BehaviorState>,
        ui: &mut bevy_inspector_egui::egui::Ui,
        type_registry: &bevy::reflect::TypeRegistryArc,
    ) {
        behavior_ui_readonly!(self, fail, state, ui, type_registry);
        behavior_ui_readonly!(self, duration, state, ui, type_registry);
        match state {
            Some(_) => {
                property_ui_readonly!(self, start, state, ui, type_registry);
                property_ui_readonly!(self, ticks, state, ui, type_registry);
            }
            _ => {}
        }
    }
}

pub fn run(
    time: Res<Time>,
    mut commands: Commands,
    mut waits: Query<
        (Entity, &mut Wait, &BehaviorNode, Option<&BehaviorStarted>),
        BehaviorRunQuery,
    >,
    mut scripts: ScriptQueries,
) {
    for (entity, mut wait, node, started) in &mut waits {
        if let BehaviorPropValue::None = wait.fail.value {
            let result = wait.fail.fetch(node, &mut scripts);
            if let Some(Err(err)) = result {
                error!("Script errored: {:?}", err);
                commands.entity(entity).insert(BehaviorFailure);
                continue;
            }
        }

        if let BehaviorPropValue::None = wait.duration.value {
            let result = wait.duration.fetch(node, &mut scripts);
            if let Some(Err(err)) = result {
                error!("Script errored: {:?}", err);
                commands.entity(entity).insert(BehaviorFailure);
                continue;
            }
        }

        if let (BehaviorPropValue::Some(wait_fail), BehaviorPropValue::Some(wait_duration)) =
            (&wait.fail.value.clone(), &wait.duration.value.clone())
        {
            let elapsed = time.elapsed_seconds_f64();
            wait.ticks += 1;
            if started.is_some() {
                wait.start = elapsed;
            }
            if elapsed - wait.start > wait_duration - f64::EPSILON {
                if *wait_fail {
                    commands.entity(entity).insert(BehaviorFailure);
                } else {
                    commands.entity(entity).insert(BehaviorSuccess);
                }
            }
        }
    }
}
