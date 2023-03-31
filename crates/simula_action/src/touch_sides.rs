use bevy::{
    input::touch::TouchPhase,
    prelude::*,
    utils::{HashMap, HashSet},
    window::PrimaryWindow,
};
use bevy_egui::{EguiContexts, EguiSet};

use crate::{Action, ActionAxis};

#[derive(Debug, Clone, Copy, Reflect, FromReflect, PartialEq, Eq, Hash)]
pub enum TouchSide {
    Left,
    Right,
}

#[derive(Debug, Clone, Copy, Reflect, FromReflect, PartialEq, Eq, Hash)]
pub enum TouchAxis {
    PositiveX,
    NegativeX,
    PositiveY,
    NegativeY,
    // TODO: Drag support similar to mouse scroll
    // Z,
}

#[derive(Debug, Clone, Copy, Reflect, FromReflect, PartialEq, Eq, Hash)]
pub enum TouchSideAxis {
    Left(TouchAxis),
    Right(TouchAxis),
}

#[derive(Resource, Default, Debug, Clone, Reflect, FromReflect)]
pub struct FingerSidesOnScreen {
    pub left: Option<u64>,
    pub right: Option<u64>,
}

pub fn touch_sides_system(
    mut egui_context: EguiContexts,
    mut touch_event: EventReader<TouchInput>,
    mut fingers_sides_on_screen: ResMut<FingerSidesOnScreen>,
    mut touch_sides_actions: Query<&mut Action<TouchSide>>,
    window: Query<&Window, With<PrimaryWindow>>,
) {
    let window = window.single();
    if egui_context.ctx_mut().wants_pointer_input() {
        for mut action in touch_sides_actions.iter_mut() {
            action.reset_all();
        }
        return;
    }
    for mut action in touch_sides_actions.iter_mut() {
        action.clear();
    }
    for event in touch_event.iter() {
        for mut action in touch_sides_actions.iter_mut() {
            let touch = if event.position.x < window.width() / 2. {
                TouchSide::Left
            } else {
                TouchSide::Right
            };
            match event.phase {
                TouchPhase::Started => {
                    match touch {
                        TouchSide::Left => {
                            if fingers_sides_on_screen.left.is_some() {
                                continue;
                            }
                            fingers_sides_on_screen.left = Some(event.id);
                        }
                        TouchSide::Right => {
                            if fingers_sides_on_screen.right.is_some() {
                                continue;
                            }
                            fingers_sides_on_screen.right = Some(event.id);
                        }
                    }
                    action.enter(touch);
                }
                TouchPhase::Ended | TouchPhase::Cancelled => {
                    if touch == TouchSide::Left {
                        fingers_sides_on_screen.left = None;
                    } else {
                        fingers_sides_on_screen.right = None;
                    }
                    action.exit(touch);
                }
                _ => {}
            }
        }
    }
}

pub fn touch_side_axis_system(
    mut egui_context: EguiContexts,
    touches: Res<Touches>,
    mut touch_side_axis_actions: Query<&mut ActionAxis<TouchSideAxis>>,
    finger_sides_on_screen: Res<FingerSidesOnScreen>,
) {
    if egui_context.ctx_mut().wants_pointer_input() {
        info!("Egui wants pointer input");
        return;
    };
    let mut exy_left = Vec2::new(0., 0.);
    let mut exy_right = Vec2::new(0., 0.);
    for mut action_axis in touch_side_axis_actions.iter_mut() {
        set_left_side_actions(&mut action_axis, exy_left);
        set_right_side_actions(&mut action_axis, exy_right);
    }
    for touch in touches.iter() {
        if finger_sides_on_screen.left == Some(touch.id()) {
            exy_left += touch.delta() * 0.01;
        }
        if finger_sides_on_screen.right == Some(touch.id()) {
            exy_right += touch.delta() * 0.01;
        }
    }
    for mut action_axis in touch_side_axis_actions.iter_mut() {
        if finger_sides_on_screen.left.is_some() {
            set_left_side_actions(&mut action_axis, exy_left);
        }
        if finger_sides_on_screen.right.is_some() {
            set_right_side_actions(&mut action_axis, exy_right);
        }
    }
}

fn set_left_side_actions(action_axis: &mut ActionAxis<TouchSideAxis>, exy: Vec2) {
    action_axis.set(TouchSideAxis::Left(TouchAxis::PositiveX), exy.x);
    action_axis.set(TouchSideAxis::Left(TouchAxis::NegativeX), -exy.x);
    action_axis.set(TouchSideAxis::Left(TouchAxis::PositiveY), exy.y);
    action_axis.set(TouchSideAxis::Left(TouchAxis::NegativeY), -exy.y);
}

fn set_right_side_actions(action_axis: &mut ActionAxis<TouchSideAxis>, exy: Vec2) {
    action_axis.set(TouchSideAxis::Right(TouchAxis::PositiveX), exy.x);
    action_axis.set(TouchSideAxis::Right(TouchAxis::NegativeX), -exy.x);
    action_axis.set(TouchSideAxis::Right(TouchAxis::PositiveY), exy.y);
    action_axis.set(TouchSideAxis::Right(TouchAxis::NegativeY), -exy.y);
}

pub struct TouchSidesPlugin;

impl Plugin for TouchSidesPlugin {
    fn build(&self, app: &mut App) {
        app.register_type::<ActionAxis<TouchSideAxis>>()
            .register_type::<ActionAxis<TouchSideAxis>>()
            .register_type::<Action<TouchSide>>()
            .register_type::<HashSet<TouchSide>>()
            .register_type::<HashMap<TouchSideAxis, f32>>()
            .register_type::<HashMap<TouchSideAxis, f32>>()
            .register_type::<FingerSidesOnScreen>()
            .init_resource::<FingerSidesOnScreen>()
            .add_systems(
                (touch_sides_system, touch_side_axis_system)
                    .after(EguiSet::ProcessInput)
                    .before(EguiSet::BeginFrame)
                    .in_base_set(CoreSet::PreUpdate),
            );
    }
}
