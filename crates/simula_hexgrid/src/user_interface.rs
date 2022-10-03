use bevy::prelude::*;
use bevy_egui::*;

use bevy_inspector_egui::bevy_egui;
use rand::Rng;

use crate::{hexgrid::{RenderAction, RenderPathEvent, RenderTile, PathFind, hexagon_pathfinder},
            pathfinding::HexOrientation::{self}};

pub fn ui_render_next_tiles(
    mut egui_ctx: ResMut<EguiContext>,
    render_path_event: EventWriter<RenderPathEvent>,
    render_tile: ResMut<RenderTile>,
) {
    render_next_tiles(&mut egui_ctx, render_path_event, render_tile);
}

pub fn render_next_tiles(
    egui_ctx: &mut ResMut<EguiContext>,
    mut render_path_event: EventWriter<RenderPathEvent>,
    mut render_tile: ResMut<RenderTile>,
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
                    render_tile.render_min_column,
                    render_tile.render_max_column,
                    render_tile.render_min_row,
                    render_tile.render_max_row
                ));
            });
            ui.horizontal(|ui| {
                if ui.button("32").clicked() {
                    render_tile.render_min_row = 0;
                    render_tile.render_max_row = 31;
                    render_tile.render_min_column = 0;
                    render_tile.render_max_column = 31;
                    render_tile.render_size = 31;
                    render_path_event.send(RenderPathEvent {
                        value: RenderAction::Rerender,
                    });
                }
                if ui.button("64").clicked() {
                    render_tile.render_min_row = 0;
                    render_tile.render_max_row = 63;
                    render_tile.render_min_column = 0;
                    render_tile.render_max_column = 63;
                    render_tile.render_size = 63;
                    render_path_event.send(RenderPathEvent {
                        value: RenderAction::Rerender,
                    });
                }
                if ui.button("128").clicked() {
                    render_tile.render_min_row = 0;
                    render_tile.render_max_row = 127;
                    render_tile.render_min_column = 0;
                    render_tile.render_max_column = 127;
                    render_tile.render_size = 127;
                    render_path_event.send(RenderPathEvent {
                        value: RenderAction::Rerender,
                    });
                }
                if ui.button("256").clicked() {
                    render_tile.render_min_row = 0;
                    render_tile.render_max_row = 255;
                    render_tile.render_min_column = 0;
                    render_tile.render_max_column = 255;
                    render_tile.render_size = 255;
                    render_path_event.send(RenderPathEvent {
                        value: RenderAction::Rerender,
                    });
                }
                if ui.button("512").clicked() {
                    render_tile.render_min_row = 0;
                    render_tile.render_max_row = 511;
                    render_tile.render_min_column = 0;
                    render_tile.render_max_column = 511;
                    render_tile.render_size = 511;
                    render_path_event.send(RenderPathEvent {
                        value: RenderAction::Rerender,
                    });
                }
                if ui.button("1024").clicked() {
                    render_tile.render_min_row = 0;
                    render_tile.render_max_row = 1023;
                    render_tile.render_min_column = 0;
                    render_tile.render_max_column = 1023;
                    render_tile.render_size = 1023;
                    render_path_event.send(RenderPathEvent {
                        value: RenderAction::Rerender,
                    });
                }
            });
            ui.horizontal(|ui| {
                ui.label("Go to tile:".to_string());
                ui.add(
                    egui::DragValue::new(&mut render_tile.tile_coord_z)
                        .clamp_range::<i32>(0..=2048),
                );
                ui.add(
                    egui::DragValue::new(&mut render_tile.tile_coord_x)
                        .clamp_range::<i32>(0..=2048),
                );
                if ui.button("Enter").clicked() {
                    render_tile.render_min_row =
                        render_tile.tile_coord_x - render_tile.render_size / 2;
                    render_tile.render_max_row =
                        render_tile.render_min_row + render_tile.render_size;
                    render_tile.render_min_column =
                        render_tile.tile_coord_z - render_tile.render_size / 2;
                    render_tile.render_max_column =
                        render_tile.render_min_column + render_tile.render_size;
                    render_path_event.send(RenderPathEvent {
                        value: RenderAction::Rerender,
                    });
                }
                if ui.button("Random").clicked() {
                    render_tile.tile_coord_x = rand::thread_rng().gen_range(0..=2048) as i32;
                    render_tile.tile_coord_z = rand::thread_rng().gen_range(0..=2048) as i32;
                }
            });
        });
}

pub fn ui_system_pathfinding_window(
    mut egui_ctx: ResMut<EguiContext>,
    mut path_find: ResMut<PathFind>,
    mut render_tile: ResMut<RenderTile>,
) {
    let end_node = (path_find.endx, path_find.endy);

    if path_find.queue_end != end_node {
        if path_find.destination_reached == false {
            hexagon_pathfinder(&mut path_find, &mut render_tile);
        }
    } else {
        path_find.destination_reached = true;
    }
    pathfinding_window(
        &mut egui_ctx,
        &mut path_find,
        &mut render_tile,
    );
}

fn pathfinding_window(
    egui_ctx: &mut ResMut<EguiContext>,
    path_find: &mut ResMut<PathFind>,
    render_tile: &mut ResMut<RenderTile>,
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
                        egui::DragValue::new(&mut path_find.startx)
                            .clamp_range::<i32>(0..=2048),
                    );
                    ui.add(
                        egui::DragValue::new(&mut path_find.starty)
                            .clamp_range::<i32>(0..=2048),
                    );
                    if ui.button("Random").clicked() {
                        path_find.startx = rand::thread_rng().gen_range(0..=2048) as i32;
                        path_find.starty = rand::thread_rng().gen_range(0..=2048) as i32;
                    }
                });
                ui.horizontal(|ui| {
                    ui.label("End: ");
                    ui.add(
                        egui::DragValue::new(&mut path_find.endx).clamp_range::<i32>(0..=2048),
                    );
                    ui.add(
                        egui::DragValue::new(&mut path_find.endy).clamp_range::<i32>(0..=2048),
                    );
                    if ui.button("Random").clicked() {
                        path_find.endx = rand::thread_rng().gen_range(0..=2048) as i32;
                        path_find.endy = rand::thread_rng().gen_range(0..=2048) as i32;
                    }
                });
                ui.horizontal(|ui| {
                    egui::ComboBox::from_label("HexOrientation")
                        .selected_text(format!("{:?}", &path_find.orientation))
                        .show_ui(ui, |ui| {
                            if ui
                                .selectable_value(
                                    &mut path_find.orientation,
                                    HexOrientation::FlatTopOddUp,
                                    "FlatTopOddUp",
                                )
                                .clicked()
                            {
                                path_find.orientation = HexOrientation::FlatTopOddUp;
                            }
                            if ui
                                .selectable_value(
                                    &mut path_find.orientation,
                                    HexOrientation::FlatTopOddDown,
                                    "FlatTopOddDown",
                                )
                                .clicked()
                            {
                                path_find.orientation = HexOrientation::FlatTopOddDown;
                            }
                            if ui
                                .selectable_value(
                                    &mut path_find.orientation,
                                    HexOrientation::PointyTopOddRight,
                                    "PointyTopOddRight",
                                )
                                .clicked()
                            {
                                path_find.orientation = HexOrientation::PointyTopOddRight;
                            }
                            if ui
                                .selectable_value(
                                    &mut path_find.orientation,
                                    HexOrientation::PointyTopOddLeft,
                                    "PointyTopOddLeft",
                                )
                                .clicked()
                            {
                                path_find.orientation = HexOrientation::PointyTopOddLeft;
                            }
                        });
                });
                ui.horizontal(|ui| {
                    if ui.button("Find Best Path").clicked() {
                        path_find.destination_reached = true;
                        hexagon_pathfinder(path_find, render_tile);
                    }
                });
                ui.horizontal(|ui| {
                    ui.label("Shortest Path: ");
                    if path_find.destination_reached == true
                        && path_find.queue_end == (path_find.endx, path_find.endy)
                    {
                        ui.add(
                            egui::Label::new(format!("{:?}", path_find.shortest_highlight))
                                .wrap(true),
                        );
                    }
                    if path_find.destination_reached == false {
                        ui.add(egui::Label::new(format!("Finding Path...")));
                    }
                });
            });
        });
}