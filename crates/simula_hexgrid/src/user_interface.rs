use bevy::prelude::*;
use bevy_egui::*;

use bevy_inspector_egui::bevy_egui;
use rand::Rng;

use crate::{hexgrid::{RenderAction, RenderPathEvent, ShortestPathBuilder, NodeStartEnd, hexagon_pathfinder},
            pathfinding::HexOrientation::{self}};

pub fn ui_render_next_tiles(
    mut egui_ctx: ResMut<EguiContext>,
    render_path_event: EventWriter<RenderPathEvent>,
    shortest_path: ResMut<ShortestPathBuilder>,
) {
    render_next_tiles(&mut egui_ctx, render_path_event, shortest_path);
}

pub fn render_next_tiles(
    egui_ctx: &mut ResMut<EguiContext>,
    mut render_path_event: EventWriter<RenderPathEvent>,
    mut shortest_path: ResMut<ShortestPathBuilder>,
) {
    egui::Window::new("Render Next Tiles")
        .default_width(200.0)
        .anchor(egui::Align2::CENTER_BOTTOM, egui::Vec2::new(0.0, -50.0))
        .resizable(true)
        .vscroll(true)
        .show(egui_ctx.ctx_mut(), |ui| {
            if ui.button("UP").clicked() {
                render_path_event.send(RenderPathEvent {
                    value: RenderAction::RenderUp,
                });
            }
            if ui.button("DOWN").clicked() {
                render_path_event.send(RenderPathEvent {
                    value: RenderAction::RenderDown,
                });
            }
            if ui.button("LEFT").clicked() {
                render_path_event.send(RenderPathEvent {
                    value: RenderAction::RenderLeft,
                });
            }
            if ui.button("RIGHT").clicked() {
                render_path_event.send(RenderPathEvent {
                    value: RenderAction::RenderRight,
                });
            }
            ui.horizontal(|ui| {
                ui.label(format!(
                    "Columns: {} to {}; Rows: {} to {}",
                    shortest_path.render_min_column,
                    shortest_path.render_max_column,
                    shortest_path.render_min_row,
                    shortest_path.render_max_row
                ));
            });
            ui.horizontal(|ui| {
                if ui.button("32").clicked() {
                    shortest_path.render_min_row = 0;
                    shortest_path.render_max_row = 31;
                    shortest_path.render_min_column = 0;
                    shortest_path.render_max_column = 31;
                    shortest_path.render_size = 31;
                    render_path_event.send(RenderPathEvent {
                        value: RenderAction::Rerender,
                    });
                }
                if ui.button("64").clicked() {
                    shortest_path.render_min_row = 0;
                    shortest_path.render_max_row = 63;
                    shortest_path.render_min_column = 0;
                    shortest_path.render_max_column = 63;
                    shortest_path.render_size = 63;
                    render_path_event.send(RenderPathEvent {
                        value: RenderAction::Rerender,
                    });
                }
                if ui.button("128").clicked() {
                    shortest_path.render_min_row = 0;
                    shortest_path.render_max_row = 127;
                    shortest_path.render_min_column = 0;
                    shortest_path.render_max_column = 127;
                    shortest_path.render_size = 127;
                    render_path_event.send(RenderPathEvent {
                        value: RenderAction::Rerender,
                    });
                }
                if ui.button("256").clicked() {
                    shortest_path.render_min_row = 0;
                    shortest_path.render_max_row = 255;
                    shortest_path.render_min_column = 0;
                    shortest_path.render_max_column = 255;
                    shortest_path.render_size = 255;
                    render_path_event.send(RenderPathEvent {
                        value: RenderAction::Rerender,
                    });
                }
                if ui.button("512").clicked() {
                    shortest_path.render_min_row = 0;
                    shortest_path.render_max_row = 511;
                    shortest_path.render_min_column = 0;
                    shortest_path.render_max_column = 511;
                    shortest_path.render_size = 511;
                    render_path_event.send(RenderPathEvent {
                        value: RenderAction::Rerender,
                    });
                }
                if ui.button("1024").clicked() {
                    shortest_path.render_min_row = 0;
                    shortest_path.render_max_row = 1023;
                    shortest_path.render_min_column = 0;
                    shortest_path.render_max_column = 1023;
                    shortest_path.render_size = 1023;
                    render_path_event.send(RenderPathEvent {
                        value: RenderAction::Rerender,
                    });
                }
            });
            ui.horizontal(|ui| {
                ui.label("Go to tile:".to_string());
                ui.add(
                    egui::DragValue::new(&mut shortest_path.tile_coord_z)
                        .clamp_range::<i32>(0..=2048),
                );
                ui.add(
                    egui::DragValue::new(&mut shortest_path.tile_coord_x)
                        .clamp_range::<i32>(0..=2048),
                );
                if ui.button("Enter").clicked() {
                    shortest_path.render_min_row =
                        shortest_path.tile_coord_x - shortest_path.render_size / 2;
                    shortest_path.render_max_row =
                        shortest_path.render_min_row + shortest_path.render_size;
                    shortest_path.render_min_column =
                        shortest_path.tile_coord_z - shortest_path.render_size / 2;
                    shortest_path.render_max_column =
                        shortest_path.render_min_column + shortest_path.render_size;
                    render_path_event.send(RenderPathEvent {
                        value: RenderAction::Rerender,
                    });
                }
                if ui.button("Random").clicked() {
                    shortest_path.tile_coord_x = rand::thread_rng().gen_range(0..=2048) as i32;
                    shortest_path.tile_coord_z = rand::thread_rng().gen_range(0..=2048) as i32;
                }
            });
        });
}

pub fn ui_system_pathfinding_window(
    mut egui_ctx: ResMut<EguiContext>,
    mut node_start_end: ResMut<NodeStartEnd>,
    mut shortest_path: ResMut<ShortestPathBuilder>,
    //mut calculate_path_event: EventWriter<CalculatePathEvent>,
) {
    let end_node = (node_start_end.endx, node_start_end.endy);

    if node_start_end.queue_end != end_node {
        if node_start_end.destination_reached == false {
            hexagon_pathfinder(&mut node_start_end, &mut shortest_path);
            //calculate_path_event.send(CalculatePathEvent);
        }
    } else {
        node_start_end.destination_reached = true;
    }
    pathfinding_window(
        &mut egui_ctx,
        &mut node_start_end,
        &mut shortest_path,
        //calculate_path_event,
    );
}

fn pathfinding_window(
    egui_ctx: &mut ResMut<EguiContext>,
    node_start_end: &mut ResMut<NodeStartEnd>,
    shortest_path: &mut ResMut<ShortestPathBuilder>,
    //mut calculate_path_event: EventWriter<CalculatePathEvent>,
) {
    egui::Window::new("Pathfinding")
        .default_width(200.0)
        .anchor(egui::Align2::RIGHT_TOP, egui::Vec2::new(-25.0, 150.0))
        .resizable(false)
        .vscroll(true)
        .show(egui_ctx.ctx_mut(), |ui| {
            ui.group(|ui| {
                ui.horizontal(|ui| {
                    ui.label("Start: ");
                    ui.add(
                        egui::DragValue::new(&mut node_start_end.startx)
                            .clamp_range::<i32>(0..=2048),
                    );
                    ui.add(
                        egui::DragValue::new(&mut node_start_end.starty)
                            .clamp_range::<i32>(0..=2048),
                    );
                    if ui.button("Random").clicked() {
                        node_start_end.startx = rand::thread_rng().gen_range(0..=2048) as i32;
                        node_start_end.starty = rand::thread_rng().gen_range(0..=2048) as i32;
                    }
                });
                ui.horizontal(|ui| {
                    ui.label("End: ");
                    ui.add(
                        egui::DragValue::new(&mut node_start_end.endx).clamp_range::<i32>(0..=2048),
                    );
                    ui.add(
                        egui::DragValue::new(&mut node_start_end.endy).clamp_range::<i32>(0..=2048),
                    );
                    if ui.button("Random").clicked() {
                        node_start_end.endx = rand::thread_rng().gen_range(0..=2048) as i32;
                        node_start_end.endy = rand::thread_rng().gen_range(0..=2048) as i32;
                    }
                });
                ui.horizontal(|ui| {
                    egui::ComboBox::from_label("HexOrientation")
                        .selected_text(format!("{:?}", &node_start_end.orientation))
                        .show_ui(ui, |ui| {
                            if ui
                                .selectable_value(
                                    &mut node_start_end.orientation,
                                    HexOrientation::FlatTopOddUp,
                                    "FlatTopOddUp",
                                )
                                .clicked()
                            {
                                node_start_end.orientation = HexOrientation::FlatTopOddUp;
                            }
                            if ui
                                .selectable_value(
                                    &mut node_start_end.orientation,
                                    HexOrientation::FlatTopOddDown,
                                    "FlatTopOddDown",
                                )
                                .clicked()
                            {
                                node_start_end.orientation = HexOrientation::FlatTopOddDown;
                            }
                            if ui
                                .selectable_value(
                                    &mut node_start_end.orientation,
                                    HexOrientation::PointyTopOddRight,
                                    "PointyTopOddRight",
                                )
                                .clicked()
                            {
                                node_start_end.orientation = HexOrientation::PointyTopOddRight;
                            }
                            if ui
                                .selectable_value(
                                    &mut node_start_end.orientation,
                                    HexOrientation::PointyTopOddLeft,
                                    "PointyTopOddLeft",
                                )
                                .clicked()
                            {
                                node_start_end.orientation = HexOrientation::PointyTopOddLeft;
                            }
                        });
                });
                ui.horizontal(|ui| {
                    if ui.button("Find Best Path").clicked() {
                        //calculate_path_event.send(CalculatePathEvent);
                        node_start_end.destination_reached = true;
                        hexagon_pathfinder(node_start_end, shortest_path);
                    }
                });
                ui.horizontal(|ui| {
                    ui.label("Shortest Path: ");
                    if node_start_end.destination_reached == true
                        && node_start_end.queue_end == (node_start_end.endx, node_start_end.endy)
                    {
                        ui.add(
                            egui::Label::new(format!("{:?}", node_start_end.shortest_highlight))
                                .wrap(true),
                        );
                    }
                    if node_start_end.destination_reached == false {
                        ui.add(egui::Label::new(format!("Finding Path...")));
                    }
                });
            });
        });
}