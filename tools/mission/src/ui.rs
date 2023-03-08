use bevy::prelude::*;
use bevy_egui::{egui, EguiContexts};
use simula_viz::follow_ui::{FollowUI, FollowUIVisibility};

use crate::common::Robot;

#[derive(Component)]
pub struct RobotPanel;

pub fn follow_ui<T>(
    mut egui_context: EguiContexts,
    follow_uis: Query<(Entity, &FollowUI, &FollowUIVisibility), With<RobotPanel>>,
    robot_query: Query<(&Name, &T)>,
) where
    T: Component + Robot,
{
    for (name, robot) in robot_query.iter() {
        if let Ok((entity, follow_ui, visibility)) = follow_uis.get(robot.get_follow_ui().unwrap())
        {
            let ui_pos = visibility.screen_pos;

            egui::Window::new("Follow UI")
                .id(egui::Id::new(entity))
                .fixed_size(egui::Vec2::new(follow_ui.size.x, follow_ui.size.y))
                .fixed_pos(egui::Pos2::new(ui_pos.x, ui_pos.y))
                .collapsible(false)
                .title_bar(false)
                .show(egui_context.ctx_mut(), |ui| {
                    ui.label(name.to_string());
                    ui.label(format!("Energy: {}", robot.get_energy() as u32));
                    ui.label(format!("Money: {}", robot.get_money()));
                });
        }
    }
}
