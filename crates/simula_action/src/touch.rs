use bevy::{
    input::touch::TouchPhase,
    prelude::*,
    utils::{HashMap, HashSet},
};
use bevy_egui::{EguiContexts, EguiSet};

use crate::{Action, ActionAxis};

#[derive(Debug, Clone, Copy, Reflect, FromReflect, PartialEq, Eq, Hash)]
pub enum TouchAxis {
    PositiveX,
    NegativeX,
    PositiveY,
    NegativeY,
    // TODO: Drag support similar to mouse scroll
    // Z,
}

#[derive(Resource, Default, Debug, Clone, Reflect, FromReflect, Deref, DerefMut)]
pub struct FingersOnScreen(pub HashSet<u64>);

#[derive(Debug, Clone, Copy, Reflect, FromReflect, PartialEq, Eq, Hash)]
pub enum Touch {
    Single,
    Double,
}

pub const FINGERS_SUPPORTED: usize = 2;

pub fn touch_system(
    mut egui_context: EguiContexts,
    mut touch_event: EventReader<TouchInput>,
    mut fingers_on_screen: ResMut<FingersOnScreen>,
    mut touch_actions: Query<&mut Action<Touch>>,
) {
    if egui_context.ctx_mut().wants_pointer_input() {
        for mut action in touch_actions.iter_mut() {
            action.reset_all();
        }
        return;
    }
    for mut action in touch_actions.iter_mut() {
        action.clear();
    }
    for event in touch_event.iter() {
        for mut action in touch_actions.iter_mut() {
            let touch = match fingers_on_screen.iter().position(|&id| id == event.id) {
                Some(_) => match fingers_on_screen.len() {
                    1 => Touch::Single,
                    2 => Touch::Double,
                    _ => continue,
                },
                None => {
                    if fingers_on_screen.len() >= FINGERS_SUPPORTED {
                        continue;
                    }
                    fingers_on_screen.insert(event.id);
                    match fingers_on_screen.len() {
                        1 => Touch::Single,
                        2 => Touch::Double,
                        _ => unreachable!(),
                    }
                }
            };

            match event.phase {
                TouchPhase::Started => {
                    action.clear();
                    action.enter(touch);
                }
                TouchPhase::Ended | TouchPhase::Cancelled => {
                    fingers_on_screen.retain(|&id| id != event.id);
                    action.exit(touch);
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
        action_axis.set(TouchAxis::PositiveX, 0.);
        action_axis.set(TouchAxis::PositiveY, 0.);
        action_axis.set(TouchAxis::NegativeX, 0.);
        action_axis.set(TouchAxis::NegativeY, 0.);
    }
    for touch in touches.iter() {
        exy += touch.delta() * 0.01;
    }
    for mut action_axis in touch_axis_actions.iter_mut() {
        action_axis.set(TouchAxis::PositiveX, exy.x);
        action_axis.set(TouchAxis::PositiveY, exy.y);
        action_axis.set(TouchAxis::NegativeX, -exy.x);
        action_axis.set(TouchAxis::NegativeY, -exy.y);
    }
}

pub struct TouchPlugin;

impl Plugin for TouchPlugin {
    fn build(&self, app: &mut App) {
        app.register_type::<ActionAxis<TouchAxis>>()
            .register_type::<Action<Touch>>()
            .register_type::<HashSet<Touch>>()
            .register_type::<HashMap<TouchAxis, f32>>()
            .register_type::<FingersOnScreen>()
            .init_resource::<FingersOnScreen>()
            .add_systems(
                (touch_system, touch_axis_system)
                    .after(EguiSet::ProcessInput)
                    .before(EguiSet::BeginFrame)
                    .in_base_set(CoreSet::PreUpdate),
            );
    }
}
