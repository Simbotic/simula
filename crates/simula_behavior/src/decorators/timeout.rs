use crate::{prelude::*, property_ui_readonly};
use bevy::prelude::*;
use bevy_inspector_egui::prelude::*;
use serde::{Deserialize, Serialize};

#[derive(
    Debug, Default, Component, Reflect, Clone, Deserialize, Serialize, InspectorOptions,
)]
#[reflect(InspectorOptions)]
pub struct Timeout {
    #[serde(default)]
    pub duration: BehaviorPropGeneric<f64>,
    #[serde(skip)]
    #[reflect(ignore)]
    pub start: f64,
}

/// Timeout will fail if its child does not return within the given time limit.
impl BehaviorSpec for Timeout {
    const TYPE: BehaviorType = BehaviorType::Decorator;
    const NAME: &'static str = "Timeout";
    const ICON: &'static str = "🕓";
    const DESC: &'static str = "Fails if its child does not return within the given time limit";
}

impl BehaviorUI for Timeout {
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
            }
            _ => {}
        }
    }
}

pub fn run(
    time: Res<Time>,
    mut commands: Commands,
    mut timeouts: Query<
        (
            Entity,
            &mut Timeout,
            &mut BehaviorChildren,
            &BehaviorNode,
            Option<&BehaviorStarted>,
            Option<&BehaviorCursor>,
        ),
        (With<Timeout>, BehaviorIdleQuery),
    >,
    nodes: Query<BehaviorChildQuery, BehaviorChildQueryFilter>,
    mut scripts: ScriptQueries,
) {
    for (entity, mut timeout, children, node, started, cursor) in &mut timeouts {
        if let BehaviorPropValue::None = timeout.duration.value {
            let result = timeout.duration.fetch(node, &mut scripts);
            if let Some(Err(err)) = result {
                error!("Script errored: {:?}", err);
                commands.entity(entity).insert(BehaviorFailure);
                continue;
            }
        }

        if let BehaviorPropValue::Some(timeout_duration) = &timeout.duration.value.clone() {
            if children.len() != 1 {
                error!("Decorator node requires one child");
                commands.entity(entity).insert(BehaviorFailure);
                continue;
            }

            let elapsed = time.elapsed_seconds_f64();
            if started.is_some() {
                timeout.start = elapsed;
            }

            if elapsed - timeout.start > timeout_duration - f64::EPSILON {
                // Time limit reached, short circuit by forcing a cursor and fail
                commands
                    .entity(entity)
                    .insert(BehaviorCursor::Return)
                    .insert(BehaviorFailure);
                continue;
            }

            if cursor.is_none() {
                continue;
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
