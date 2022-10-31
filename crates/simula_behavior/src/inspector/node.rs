use crate::{
    color_hex_utils::color_from_hex, BehaviorChildQuery, BehaviorChildQueryFilter,
    BehaviorChildQueryItem, BehaviorChildren, BehaviorCursor, BehaviorFailure, BehaviorInfo,
    BehaviorNode, BehaviorParent, BehaviorPlugin, BehaviorRunQuery, BehaviorRunning,
    BehaviorSpawner, BehaviorSuccess, BehaviorTree, BehaviorType,
};
use bevy::{ecs::component::ComponentId, prelude::*};
use bevy_inspector_egui::{
    egui::{self, Rounding},
    options::EntityAttributes,
    Context, Inspectable,
};
use pretty_type_name::{pretty_type_name, pretty_type_name_str};

// let connector = egui::Shape::Circle(egui::epaint::CircleShape{
//     center: egui::Pos2::new(0.0, 0.0),
//     radius: 100.0,
//     fill: color_from_hex("#3f3f3f").unwrap(),
//     stroke: egui::Stroke::none(),
// });
// ui.painter().add(connector.clone());

#[derive(Default, Clone)]
pub struct BehaviorInspectorNodeAttributes;
//  {
//     pub name: String,
//     pub description: String,
// }

#[derive(Default, Clone)]
pub struct BehaviorInspectorNode {
    pub entity: Option<Entity>,
}

fn titlebar_color(behavior: &BehaviorNode) -> egui::Color32 {
    if behavior.typ == BehaviorType::Action {
        color_from_hex("#4284F3").unwrap()
    } else if behavior.typ == BehaviorType::Composite {
        if behavior.name == "Selector" {
            color_from_hex("#CC0100").unwrap()
        } else if behavior.name == "Sequence" {
            color_from_hex("#36980D").unwrap()
        } else {
            color_from_hex("#F3F342").unwrap()
        }
    } else if behavior.typ == BehaviorType::Decorator {
        color_from_hex("#ACA000").unwrap()
    } else {
        color_from_hex("#3f3f3f").unwrap()
    }
}

fn behavior_nodes(
    world: &mut World,
) -> QueryState<
    (
        Entity,
        &BehaviorNode,
        &BehaviorChildren,
        Option<&BehaviorRunning>,
        Option<&BehaviorFailure>,
        Option<&BehaviorSuccess>,
    ),
    With<BehaviorNode>,
> {
    world.query_filtered::<(
        Entity,
        &BehaviorNode,
        &BehaviorChildren,
        Option<&BehaviorRunning>,
        Option<&BehaviorFailure>,
        Option<&BehaviorSuccess>,
    ), With<BehaviorNode>>()
}

macro_rules! some_or_return {
    ( $e:expr ) => {
        match $e {
            Some(x) => x,
            None => return false,
        }
    };
}

impl Inspectable for BehaviorInspectorNode {
    type Attributes = BehaviorInspectorNodeAttributes;

    fn ui(&mut self, ui: &mut egui::Ui, options: Self::Attributes, context: &mut Context) -> bool {
        let mut changed = false;

        let entity = some_or_return!(self.entity);
        let world = some_or_return!(unsafe { context.world_mut() });
        let behavior_name = some_or_return!(world.get::<Name>(entity));
        let behavior_node = some_or_return!(world.get::<BehaviorNode>(entity));
        let behavior_children = some_or_return!(world.get::<BehaviorChildren>(entity));

        let behavior_running = world.get::<BehaviorRunning>(entity);
        let behavior_failure = world.get::<BehaviorFailure>(entity);
        let behavior_success = world.get::<BehaviorSuccess>(entity);

        // Node frame
        egui::Frame::none()
            .fill(egui::Color32::DARK_GRAY)
            .rounding(Rounding::same(3.0))
            .show(ui, |ui| {
                egui::Frame::none()
                    .inner_margin(egui::Vec2::new(4.0, 1.0))
                    .rounding(Rounding::same(3.0))
                    .fill(titlebar_color(&behavior_node))
                    .show(ui, |ui| {
                        // ui.scope(|ui| {
                        //     ui.visuals_mut().override_text_color = Some(egui::Color32::WHITE);
                        //     ui.horizontal(|ui| {
                        //         ui.label(behavior_name.to_string());
                        //     });
                        // });
                        ui.visuals_mut().override_text_color = Some(egui::Color32::WHITE);
                        ui.horizontal(|ui| {
                            ui.label(behavior_name.to_string());
                        });
                    });

                ui.collapsing("", |ui| {
                    if let Some(entity) = &mut self.entity {
                        let attributes = EntityAttributes { despawnable: false };
                        entity.ui(ui, attributes, context);
                    }
                });

                // ui.set_min_height(200.0);
            });

        // ui.style_mut().visuals.window_fill() = egui::Color32::from_rgb(0, 0, 0);

        // let ui_ctx = match context.ui_ctx {
        //     Some(ctx) => ctx,
        //     None => {
        //         ui.label(
        //             "Need `context.ui_ctx` for the `BehaviorInspectorNode` inspectable implementation",
        //         );
        //         return false;
        //     }
        // };

        // egui::Area::new("title").show(ui_ctx, |ui| {
        //     ui.label(pretty_type_name::<Self>());
        // });

        // ui.scope(|ui| {
        //     // ui.style_mut().

        //     ui.group(|ui| {
        //         egui::Frame::none().fill(egui::Color32::RED).show(ui, |ui| {
        //                 ui.label(pretty_type_name::<Self>());
        //                 ui.small_button("X");
        //         });
        //         // ui.group(|ui| {
        //         //     ui.horizontal(|ui| {
        //         //         ui.label(pretty_type_name::<Self>());
        //         //         ui.small_button("X");
        //         //     });
        //         // });
        //         // ui.label(pretty_type_name::<Self>());
        //         ui.set_min_height(200.0);
        //     });
        // });

        // ui.label(pretty_type_name::<Self>());

        // if let Some(entity) = self.entity {
        //     let world = context.world();
        //     if let Some(world) = world {

        //         let (_entity_location, components) = {
        //             let entity_ref = match world.get_entity(entity) {
        //                 Some(entity_ref) => entity_ref,
        //                 None => {
        //                     ui.label("Entity does not exist");
        //                     return false;
        //                 }
        //             };
        //             let entity_location = entity_ref.location();
        //             let archetype = entity_ref.archetype();

        //             let table_components = archetype.table_components().into_iter();
        //             let sparse_components = archetype.sparse_set_components().into_iter();
        //             let components: Vec<ComponentId> = table_components.chain(sparse_components).cloned().collect();

        //             (entity_location, components)
        //         };

        //         let mut components: Vec<_> = components
        //         .iter()
        //         .map(|component_id| {
        //             let component_info = world.components().get_info(*component_id).unwrap();
        //             let name = pretty_type_name_str(component_info.name());
        //             (
        //                 name,
        //                 component_info.id(),
        //                 component_info.type_id(),
        //                 component_info.layout().size(),
        //             )
        //         })
        //         .collect();

        //         for (name, component_id, type_id, size) in components.iter() {
        //             ui.label(format!("{}: {:?} ({} bytes)", name, type_id, size));
        //         }

        //         ui.separator();

        //         // for (name, component_id, type_id, size) in components.iter() {
        //         //     let mut component = world.get_component::<dyn std::any::Any>(entity, *component_id).unwrap();
        //         //     let mut component = component.downcast_mut::<dyn Inspectable>().unwrap();
        //         //     let mut attributes = Default::default();
        //         //     let mut context = Default::default();
        //         //     component.ui(ui, attributes, &mut context);
        //         // }
        //     }
        // }

        changed
    }
}
