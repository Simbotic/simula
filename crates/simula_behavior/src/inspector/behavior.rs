use crate::prelude::*;
use bevy::{prelude::*, reflect::TypeRegistry};

pub trait BehaviorUI
where
    Self: Reflect,
{
    /// ui inspector for behavior properties
    fn ui(
        &mut self,
        _state: Option<protocol::BehaviorState>,
        ui: &mut bevy_inspector_egui::egui::Ui,
        type_registry: &TypeRegistry,
    ) -> bool {
        let type_registry = type_registry.read();
        bevy_inspector_egui::reflect_inspector::ui_for_value(
            self.as_reflect_mut(),
            ui,
            &type_registry,
        )
    }

    /// ui readonly inspector for behavior properties
    fn ui_readonly(
        &self,
        _state: Option<protocol::BehaviorState>,
        ui: &mut bevy_inspector_egui::egui::Ui,
        type_registry: &TypeRegistry,
    ) {
        let type_registry = type_registry.read();
        bevy_inspector_egui::reflect_inspector::ui_for_value_readonly(
            self.as_reflect(),
            ui,
            &type_registry,
        )
    }
}
