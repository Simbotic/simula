use crate::{bevy_inspector_egui::bevy_inspector, egui, Inspector, Inspectors};
use bevy::prelude::*;

pub struct WorldInspectorPlugin;

impl Plugin for WorldInspectorPlugin {
    fn build(&self, app: &mut App) {
        app.insert_resource(WorldInspector::default())
            .add_startup_system(setup);
    }
}

#[derive(Copy, Clone, Reflect, Default, PartialEq)]
enum InspectorType {
    #[default]
    None,
    Entities,
    Resources,
    Assets,
}

#[derive(Default, Clone, Resource, Reflect)]
struct WorldInspector {
    selected: InspectorType,
}

fn item_label(item: &InspectorType) -> String {
    match item {
        InspectorType::None => "None".to_string(),
        InspectorType::Entities => "♜ Entities".to_string(),
        InspectorType::Resources => "📦 Resources".to_string(),
        InspectorType::Assets => "🎨 Assets".to_string(),
    }
}

fn menu_ui(ui: &mut egui::Ui, world: &mut World) {
    let mut world_inspector = world.resource_mut::<WorldInspector>();

    egui::menu::menu_button(ui, "🌎 World", |_ui| {});

    egui::ComboBox::from_id_source("World Inspector Selector")
        .selected_text(item_label(&world_inspector.selected))
        .show_ui(ui, |ui| {
            let selectable_behaviors = vec![
                InspectorType::None,
                InspectorType::Entities,
                InspectorType::Resources,
                InspectorType::Assets,
            ];
            for selectable_behavior in selectable_behaviors {
                ui.allocate_ui(egui::vec2(200.0, 1.0), |ui| {
                    if ui
                        .selectable_label(
                            world_inspector.selected == selectable_behavior,
                            item_label(&selectable_behavior),
                        )
                        .clicked()
                    {
                        world_inspector.selected = selectable_behavior;
                    }
                });
            }
        });
}

fn window_ui(context: &mut egui::Context, world: &mut World) {
    let show = world.resource::<WorldInspector>().selected;

    if show != InspectorType::None {
        let parent_rect = context.available_rect();
        let desired_width = 300.0;
        let desired_height = parent_rect.height() * 0.9;
        let desired_x = parent_rect.max.x - desired_width;
        let desired_y = parent_rect.min.y;

        let label = item_label(&show);
        let mut open = true;
        egui::Window::new(format!("{}", label))
            .open(&mut open)
            .default_pos(egui::Pos2::new(desired_x, desired_y))
            .default_size(egui::Vec2::new(desired_width, desired_height))
            .show(context, |ui| {
                egui::ScrollArea::vertical().show(ui, |ui| match show {
                    InspectorType::Entities => {
                        bevy_inspector::ui_for_world_entities(world, ui);
                    }
                    InspectorType::Resources => {
                        bevy_inspector::ui_for_resources(world, ui);
                    }
                    InspectorType::Assets => {
                        bevy_inspector::ui_for_all_assets(world, ui);
                    }
                    _ => {}
                });
            });
        if !open {
            let mut world_inspector = world.resource_mut::<WorldInspector>();
            world_inspector.selected = InspectorType::None;
        }
    }
}

fn setup(mut inspectors: ResMut<Inspectors>) {
    inspectors.inspectors.push(Inspector { menu_ui, window_ui });
}
