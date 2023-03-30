use bevy::{
    input::touch::TouchPhase,
    prelude::*,
    utils::{HashMap, HashSet},
    window::PrimaryWindow,
};
use bevy_egui::{EguiContexts, EguiSet};

use crate::{Action, ActionAxis};

#[derive(Debug, Clone, Copy, Reflect, FromReflect, PartialEq, Eq, Hash)]
pub enum TouchAxis {
    X,
    Y,
    // TODO: Drag support similar to mouse scroll
    // Z,
}

#[derive(Debug, Clone, Copy, Reflect, FromReflect, PartialEq, Eq, Hash)]
pub enum TouchSide {
    Left,
    Right,
}

pub fn touch_side_system(
    mut egui_context: EguiContexts,
    mut touch_event: EventReader<TouchInput>,
    mut touch_side_actions: Query<&mut Action<TouchSide>>,
    window: Query<&Window, With<PrimaryWindow>>,
) {
    let window = window.single();
    if egui_context.ctx_mut().wants_pointer_input() {
        for mut action in touch_side_actions.iter_mut() {
            action.reset_all();
        }
        return;
    }
    for mut action in touch_side_actions.iter_mut() {
        action.clear();
    }
    for event in touch_event.iter() {
        for mut action in touch_side_actions.iter_mut() {
            match event.phase {
                TouchPhase::Started => {
                    if event.position.x < (window.resolution.physical_width() / 2) as f32 {
                        action.enter(TouchSide::Left);
                    } else if event.position.x > (window.resolution.physical_width() / 2) as f32 {
                        action.enter(TouchSide::Right);
                    }
                }
                TouchPhase::Ended | TouchPhase::Cancelled => {
                    if event.position.x < (window.resolution.physical_width() / 2) as f32 {
                        action.exit(TouchSide::Left);
                    } else if event.position.x > (window.resolution.physical_width() / 2) as f32 {
                        action.exit(TouchSide::Right);
                    }
                }
                _ => {}
            }
        }
    }
}

pub fn touch_axis_system(
    mut egui_context: EguiContexts,
    touches: Res<Touches>,
    mut touch_axis_actions: Query<&mut ActionAxis<TouchAxis>>,
) {
    if egui_context.ctx_mut().wants_pointer_input() {
        info!("Egui wants pointer input");
        return;
    }
    let mut exy = Vec2::new(0., 0.);
    for mut action_axis in touch_axis_actions.iter_mut() {
        action_axis.set(TouchAxis::X, 0.);
        action_axis.set(TouchAxis::Y, 0.);
    }
    for touch in touches.iter() {
        exy += touch.delta() * 0.01;
    }
    for mut action_axis in touch_axis_actions.iter_mut() {
        action_axis.set(TouchAxis::X, exy.x);
        action_axis.set(TouchAxis::Y, exy.y);
    }
}

pub struct TouchPlugin;

impl Plugin for TouchPlugin {
    fn build(&self, app: &mut App) {
        app.register_type::<ActionAxis<TouchAxis>>()
            .register_type::<Action<TouchSide>>()
            .register_type::<HashSet<TouchSide>>()
            .register_type::<HashMap<TouchAxis, f32>>()
            .add_systems(
                (touch_side_system, touch_axis_system)
                    .after(EguiSet::ProcessInput)
                    .before(EguiSet::BeginFrame)
                    .in_base_set(CoreSet::PreUpdate),
            );
    }
}
