use crate::prelude::*;
use bevy::{prelude::*, reflect::TypeRegistry};

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
        _label: Option<&str>,
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
