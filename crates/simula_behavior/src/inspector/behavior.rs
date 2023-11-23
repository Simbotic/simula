use crate::prelude::*;
use bevy::{prelude::*, reflect::TypeRegistryArc};

pub trait BehaviorUI
where
    Self: Reflect,
{
    /// ui inspector for behavior properties
    fn ui(
        &mut self,
        _label: Option<&str>,
        _state: Option<protocol::BehaviorState>,
        ui: &mut bevy_inspector_egui::egui::Ui,
        type_registry: &TypeRegistryArc,
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
        _label: Option<&str>,
        _state: Option<protocol::BehaviorState>,
        ui: &mut bevy_inspector_egui::egui::Ui,
        type_registry: &TypeRegistryArc,
    ) {
        let type_registry = type_registry.read();
        bevy_inspector_egui::reflect_inspector::ui_for_value_readonly(
            self.as_reflect(),
            ui,
            &type_registry,
        )
    }

    fn add_property_readonly(
        value: &dyn Reflect,
        label: &str,
        _state: Option<protocol::BehaviorState>,
        ui: &mut bevy_inspector_egui::egui::Ui,
        type_registry: &TypeRegistryArc,
    ) {
        let type_registry = type_registry.read();
        ui.horizontal(|ui| {
            ui.label(label);
            bevy_inspector_egui::reflect_inspector::ui_for_value_readonly(
                value,
                ui,
                &type_registry,
            );
        });
    }
}

#[macro_export]
macro_rules! behavior_ui {
    ($s:expr, $field:ident, $state:expr, $ui:expr, $type_registry:expr) => {
        $s.$field
            .ui(Some(stringify!($field)), $state, $ui, $type_registry)
    };
}

#[macro_export]
macro_rules! behavior_ui_readonly {
    ($s:expr, $field:ident, $state:expr, $ui:expr, $type_registry:expr) => {
        $s.$field
            .ui_readonly(Some(stringify!($field)), $state, $ui, $type_registry)
    };
}

#[macro_export]
macro_rules! property_ui_readonly {
    ($s:expr, $field:ident, $state:expr, $ui:expr, $type_registry:expr) => {
        Self::add_property_readonly(
            $s.$field.as_reflect(),
            stringify!($field),
            $state,
            $ui,
            $type_registry,
        )
    };
}
