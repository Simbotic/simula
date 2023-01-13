use bevy::prelude::*;
use bevy_egui::{egui, EguiContext};
use simula_viz::follow_ui::{FollowUI, FollowUIVisibility};

#[derive(Component)]
pub struct RobotPanel;

pub fn follow_ui(
    mut egui_context: ResMut<EguiContext>,
    follow_uis: Query<(Entity, &FollowUI, &FollowUIVisibility), With<RobotPanel>>,
) {
    for (entity, follow_ui, visibility) in follow_uis.iter() {
        let ui_pos = visibility.screen_pos;

        egui::Window::new("Follow UI")
            .id(egui::Id::new(entity))
            .fixed_size(egui::Vec2::new(follow_ui.size.x, follow_ui.size.y))
            .fixed_pos(egui::Pos2::new(ui_pos.x, ui_pos.y))
            .collapsible(false)
            .title_bar(false)
            .show(egui_context.ctx_mut(), |ui| {
                ui.label("Hello World");
            });
    }
}
