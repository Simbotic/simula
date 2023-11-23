use crate::{prelude::*, property_ui_readonly};
use bevy::prelude::*;
use bevy_inspector_egui::prelude::*;
use serde::{Deserialize, Serialize};

/// Delay will delay the execution of its child.
#[derive(
    Debug, Default, Component, Reflect, Clone, Deserialize, Serialize, InspectorOptions,
)]
#[reflect(InspectorOptions)]
pub struct Delay {
    #[serde(default)]
    pub duration: BehaviorPropGeneric<f64>,
    #[serde(skip)]
    pub start: f64,
    #[serde(skip)]
    pub ticks: u64,
}

impl BehaviorSpec for Delay {
    const TYPE: BehaviorType = BehaviorType::Decorator;
    const NAME: &'static str = "Delay";
    const ICON: &'static str = "⌛";
    const DESC: &'static str = "Delays the execution of its child";
}

impl BehaviorUI for Delay {
    fn ui(
        &mut self,
        _label: Option<&str>,
        state: Option<protocol::BehaviorState>,
        ui: &mut bevy_inspector_egui::egui::Ui,
        type_registry: &bevy::reflect::TypeRegistryArc,
    ) -> bool {
        let mut changed = false;
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
    mut delays: Query<
        (
            Entity,
            &mut Delay,
            &mut BehaviorChildren,
            &BehaviorNode,
            Option<&BehaviorStarted>,
        ),
        (With<Delay>, BehaviorRunQuery),
    >,
    nodes: Query<BehaviorChildQuery, BehaviorChildQueryFilter>,
    mut scripts: ScriptQueries,
) {
    for (entity, mut delay, children, node, started) in &mut delays {
        if let BehaviorPropValue::None = delay.duration.value {
            let result = delay.duration.fetch(node, &mut scripts);
            if let Some(Err(err)) = result {
                error!("Script errored: {:?}", err);
                commands.entity(entity).insert(BehaviorFailure);
                continue;
            }
        }

        if let BehaviorPropValue::Some(delay_duration) = &delay.duration.value.clone() {
            delay.ticks += 1;

            if children.len() != 1 {
                error!("Decorator node requires one child");
                commands.entity(entity).insert(BehaviorFailure);
                continue;
            }

            let elapsed = time.elapsed_seconds_f64();
            if started.is_some() {
                delay.start = elapsed;
            }

            if elapsed - delay.start < delay_duration + f64::EPSILON {
                continue; // We're still in delay, so don't do anything yet.
            }

            let child_entity = children[0]; // Safe because we checked for empty
            if let Ok(BehaviorChildQueryItem {
                child_entity,
                child_parent: _,
                child_failure,
                child_success,
                child_running: _,
            }) = nodes.get(child_entity)
            {
                // Child failed, so we fail
                if child_failure.is_some() {
                    commands.entity(entity).insert(BehaviorFailure);
                }
                // Child succeeded, so we succeed
                else if child_success.is_some() {
                    commands.entity(entity).insert(BehaviorSuccess);
                }
                // Child is ready, pass on cursor
                else {
                    commands.entity(entity).remove::<BehaviorCursor>();
                    commands
                        .entity(child_entity)
                        .insert(BehaviorCursor::Delegate);
                }
            }
        }
    }
}
