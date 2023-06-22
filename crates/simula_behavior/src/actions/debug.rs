use crate::prelude::*;
use bevy::prelude::*;
use bevy_inspector_egui::prelude::*;
use serde::{Deserialize, Serialize};
use std::borrow::Cow;

#[derive(
    Debug, Default, Component, Reflect, FromReflect, Clone, Deserialize, Serialize, InspectorOptions,
)]
#[reflect(InspectorOptions)]
pub struct Debug {
    #[serde(default)]
    pub message: Cow<'static, str>,
    #[serde(default)]
    pub fail: bool,
    #[serde(default)]
    #[inspector(min = 0.0, max = f64::MAX)]
    pub duration: f64,
    #[serde(skip)]
    pub start: f64,
    #[serde(skip)]
    pub ticks: u64,
}

impl BehaviorInfo for Debug {
    const TYPE: BehaviorType = BehaviorType::Action;
    const NAME: &'static str = "Debug";
    const ICON: &'static str = "👁";
    const DESC: &'static str = "Display a debug message and complete with success or failure";
}

pub fn run(
    time: Res<Time>,
    mut commands: Commands,
    mut debug_actions: Query<
        (Entity, &mut Debug, Option<&Name>, Option<&BehaviorStarted>),
        BehaviorRunQuery,
    >,
) {
    for (entity, mut debug_action, name, started) in &mut debug_actions {
        let elapsed = time.elapsed_seconds_f64();
        debug_action.ticks += 1;
        if started.is_some() {
            debug_action.start = elapsed;
            let name = name.map(|name| name.as_str()).unwrap_or("");
            info!("[{}:{}] {}", entity.index(), name, debug_action.message);
        }
        if elapsed - debug_action.start > debug_action.duration - f64::EPSILON {
            if debug_action.fail {
                commands.entity(entity).insert(BehaviorFailure);
            } else {
                commands.entity(entity).insert(BehaviorSuccess);
            }
        }
    }
}
