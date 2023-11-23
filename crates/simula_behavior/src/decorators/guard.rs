use crate::prelude::*;
use bevy::prelude::*;
use serde::{Deserialize, Serialize};

/// Guard evals a script to control the flow of execution. If the script returns
/// `true`, the child is executed. If the script returns `false`, the child is
/// not executed. The Scope of the script should be at the tree entity.
#[derive(
    Debug, Deref, DerefMut, Component, Reflect, Clone, Deserialize, Serialize,
)]
pub struct Guard {
    #[serde(default)]
    pub condition: BehaviorPropGeneric<bool>,
}

impl Default for Guard {
    fn default() -> Self {
        Self {
            condition: BehaviorPropGeneric {
                prop: BehaviorEval::Value(true),
                ..default()
            },
        }
    }
}

impl BehaviorSpec for Guard {
    const TYPE: BehaviorType = BehaviorType::Decorator;
    const NAME: &'static str = "Guard";
    const ICON: &'static str = "🚫";
    const DESC: &'static str =
        "Guard evals a script to control the flow of execution. If the script returns \
        `true`, the child is executed. If the script returns `false`, the child is \
        not executed. The Scope of the script should be at the tree entity.";
}

impl BehaviorUI for Guard {
    fn ui(
        &mut self,
        _label: Option<&str>,
        state: Option<protocol::BehaviorState>,
        ui: &mut bevy_inspector_egui::egui::Ui,
        type_registry: &bevy::reflect::TypeRegistryArc,
    ) -> bool {
        let mut changed = false;
        changed |= behavior_ui!(self, condition, state, ui, type_registry);
        changed
    }

    fn ui_readonly(
        &self,
        _label: Option<&str>,
        state: Option<protocol::BehaviorState>,
        ui: &mut bevy_inspector_egui::egui::Ui,
        type_registry: &bevy::reflect::TypeRegistryArc,
    ) {
        behavior_ui_readonly!(self, condition, state, ui, type_registry);
    }
}

pub fn run(
    mut commands: Commands,
    mut guards: Query<
        (
            Entity,
            &BehaviorChildren,
            &mut Guard,
            &BehaviorNode,
            Option<&BehaviorStarted>,
        ),
        BehaviorRunQuery,
    >,
    nodes: Query<BehaviorChildQuery, BehaviorChildQueryFilter>,
    mut scripts: ScriptQueries,
) {
    for (entity, children, mut guard, node, started) in &mut guards {
        if let BehaviorPropValue::None = guard.condition.value {
            let result = guard.condition.fetch(node, &mut scripts);
            if let Some(Err(err)) = result {
                error!("Script errored: {:?}", err);
                commands.entity(entity).insert(BehaviorFailure);
                continue;
            }
        }

        if children.len() != 1 {
            error!("Decorator node requires one child");
            commands.entity(entity).insert(BehaviorFailure);
            continue;
        }

        if started.is_some() {
            guard.condition.value = BehaviorPropValue::None;
        }

        let _ = guard.condition.fetch(node, &mut scripts);
        let child_entity = children[0]; // Safe because we checked for empty
        if let Ok(BehaviorChildQueryItem {
            child_entity,
            child_parent: _,
            child_failure,
            child_success,
            child_running: _,
        }) = nodes.get(child_entity)
        {
            // Child failed, we fail
            if child_failure.is_some() {
                commands.entity(entity).insert(BehaviorFailure);
            }
            // Child succeeded, so we succeed
            else if child_success.is_some() {
                commands.entity(entity).insert(BehaviorSuccess);
            }
            // Child is ready, eval script to see if we should pass on cursor
            else {
                match &guard.condition.value {
                    BehaviorPropValue::Some(true) => {
                        // Script returned true, so let the child run
                        commands.entity(entity).remove::<BehaviorCursor>();
                        commands
                            .entity(child_entity)
                            .insert(BehaviorCursor::Delegate);
                    }
                    BehaviorPropValue::Some(false) => {
                        // Script returned false, so we fail
                        commands.entity(entity).insert(BehaviorFailure);
                    }
                    BehaviorPropValue::Err(err) => {
                        // Script errored, so we fail
                        error!("Script errored: {:?}", err);
                        commands.entity(entity).insert(BehaviorFailure);
                    }
                    BehaviorPropValue::None => {
                        // Script is still busy
                    }
                };
            }
        }
    }
}
