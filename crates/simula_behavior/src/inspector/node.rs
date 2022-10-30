use crate::{
    color_hex_utils::color_from_hex, BehaviorChildQuery, BehaviorChildQueryFilter,
    BehaviorChildQueryItem, BehaviorChildren, BehaviorCursor, BehaviorFailure, BehaviorInfo,
    BehaviorParent, BehaviorPlugin, BehaviorRunQuery, BehaviorRunning, BehaviorSpawner,
    BehaviorSuccess, BehaviorTree, BehaviorType,
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

fn titlebar_color<T>(info: &T) -> egui::Color32
where
    T: BehaviorInfo,
{
    if T::TYPE == BehaviorType::Action {
        color_from_hex("#3f3f3f").unwrap()
    } else if T::TYPE == BehaviorType::Composite {
        color_from_hex("#3f3f3f").unwrap()
    } else if T::TYPE == BehaviorType::Decorator {
        color_from_hex("#3f3f3f").unwrap()
    } else {
        color_from_hex("#3f3f3f").unwrap()
    }
}



impl Inspectable for BehaviorInspectorNode {
    type Attributes = BehaviorInspectorNodeAttributes;

    fn ui(&mut self, ui: &mut egui::Ui, options: Self::Attributes, context: &mut Context) -> bool {
        let mut changed = false;

        // let info = if let Some(entity) = self.entity {
        //     if let Ok(info) = context.world().get::<BehaviorInfo>(entity) {
        //         info
        //     } else {
        //         return changed;
        //     }
        // } else {
        //     return changed;
        // };

        // Node frame
        egui::Frame::none()
            .fill(egui::Color32::DARK_GRAY)
            .rounding(Rounding::same(3.0))
            .show(ui, |ui| {
                egui::Frame::none()
                    .inner_margin(egui::Vec2::new(4.0, 1.0))
                    .rounding(Rounding::same(3.0))
                    .fill(egui::Color32::RED)
                    .show(ui, |ui| {
                        ui.horizontal(|ui| {
                            ui.label("Behavior");
                        });
                    });

                ui.collapsing("Entity", |ui| {
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
