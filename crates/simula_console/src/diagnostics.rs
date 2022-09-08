use anyhow::Result;
use bevy::render::once_cell;

use bevy::{
    diagnostic::{Diagnostics, FrameTimeDiagnosticsPlugin},
    prelude::*,
    render::{render_resource::WgpuAdapterInfo, renderer::RenderDevice},
};

use bevy_egui::{egui, EguiContext};

use enum_iterator::IntoEnumIterator;
use nvml_wrapper::struct_wrappers::device::MemoryInfo;
use serde::{Deserialize, Serialize};

use nvml_wrapper::Nvml;
use once_cell::sync::Lazy;

use crate::loader::DiagnosticsLoaderPlugin;

#[allow(missing_docs)]
#[derive(Reflect, Debug, Copy, Clone, PartialEq, Serialize, Deserialize, IntoEnumIterator)]
pub enum DiagnosticPowerPreference {
    LowPower,
    HighPerformance,
}

#[allow(missing_docs)]
#[derive(Reflect, Debug, Copy, Clone, PartialEq, Serialize, Deserialize, IntoEnumIterator)]
pub enum DiagnosticLimits {
    DownlevelDefaults,
    DownlevelWebgl2Defaults,
    Default,
}

#[derive(Reflect, Debug, Copy, Clone, PartialEq, Serialize, Deserialize, IntoEnumIterator)]
pub enum DiagnosticWgpuSettingsPriority {
    Compatibility,
    Functionality,
    WebGL2,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct DiagnosticWgpuSettings {
    pub power_preference: DiagnosticPowerPreference,
    pub priority: DiagnosticWgpuSettingsPriority,
    pub limits: DiagnosticLimits,
}

static NVML: Lazy<Nvml> = Lazy::new(|| {
    fn gpu_device() -> Result<Nvml> {
        let nvml = Nvml::init()?;
        Ok(nvml)
    }
    gpu_device().unwrap()
});

pub struct DiagnosticParameters {
    pub framerate: f64,
}

impl Default for DiagnosticParameters {
    fn default() -> Self {
        DiagnosticParameters { framerate: 0.0 }
    }
}
pub struct WindowAnchor {
    pub anchor_align: egui::Align2,
    pub anchor_offset: egui::Vec2,
}

pub fn ui_system_diagnostics(
    mut ctx: ResMut<EguiContext>,
    diag_parameters: ResMut<DiagnosticParameters>,
    wgpu_parameters: ResMut<WgpuAdapterInfo>,
    render_device: ResMut<RenderDevice>,
    diagnostics_settings: Res<SimulaDiagnosticsSettings>,
) {
    let mut windows = egui::Window::new("Diagnostics")
        .collapsible(diagnostics_settings.collapsible)
        .vscroll(diagnostics_settings.vscroll)
        .drag_bounds(egui::Rect::EVERYTHING)
        .resizable(diagnostics_settings.resizable)
        .auto_sized();

    if let Some(anchor) = &diagnostics_settings.anchor {
        windows = windows.anchor(anchor.anchor_align, anchor.anchor_offset);
    }

    windows.show(ctx.ctx_mut(), |ui| {
        ui.group(|ui| {
            ui.label("Nvidia Memory Info: ");
            ui.add(
                egui::Label::new(format!(
                    "Used: {:?} MB",
                    memory_info().unwrap().used / 1000000
                ))
                .wrap(true),
            );
            ui.add(
                egui::Label::new(format!(
                    "Free: {:?} MB",
                    memory_info().unwrap().free / 1000000
                ))
                .wrap(true),
            );
            ui.add(
                egui::Label::new(format!(
                    "Total: {:?} MB",
                    memory_info().unwrap().total / 1000000
                ))
                .wrap(true),
            );
        });
        ui.horizontal(|ui| {
            ui.label("Framerate: ");
            ui.add(egui::Label::new(format!("{:.05}", diag_parameters.framerate)).wrap(true));
        });
        ui.collapsing("WGPU Info", |ui| {
            ui.horizontal(|ui| {
                ui.label("Name: ");
                ui.add(egui::Label::new(format!("{:?}", wgpu_parameters.name)).wrap(true));
            });
            ui.horizontal(|ui| {
                ui.label("Vendor: ");
                ui.add(egui::Label::new(format!("{:?}", wgpu_parameters.vendor)).wrap(true));
            });
            ui.horizontal(|ui| {
                ui.label("Device: ");
                ui.add(egui::Label::new(format!("{:?}", wgpu_parameters.device)).wrap(true));
            });
            ui.horizontal(|ui| {
                ui.label("Device Type: ");
                ui.add(egui::Label::new(format!("{:?}", wgpu_parameters.device_type)).wrap(true));
            });
            ui.horizontal(|ui| {
                ui.label("Backend: ");
                ui.add(egui::Label::new(format!("{:?}", wgpu_parameters.backend)).wrap(true));
            });
        });
        ui.collapsing("Render Device", |ui| {
            ui.horizontal(|ui| {
                ui.label("Features: ");
                ui.add(egui::Label::new(format!("{:?}", render_device.features())).wrap(true));
            });
            ui.horizontal(|ui| {
                ui.label("Limits: ");
                ui.add(egui::Label::new(format!("{:?}", render_device.limits())).wrap(true));
            });
        });
    });
}

fn memory_info() -> Result<MemoryInfo> {
    let device = Lazy::force(&NVML).device_by_index(0)?;
    let memory_info = device.memory_info()?;
    Ok(memory_info)
}

pub fn framerate_update_system(
    diagnostics: Res<Diagnostics>,
    mut diag_parameters: ResMut<DiagnosticParameters>,
) {
    if let Some(fps) = diagnostics.get(FrameTimeDiagnosticsPlugin::FPS) {
        if let Some(average) = fps.average() {
            diag_parameters.framerate = average;
        }
    }
}
pub struct SimulaDiagnosticsSettings {
    pub config_path: String,
    pub collapsible: bool,
    pub vscroll: bool,
    pub resizable: bool,
    pub anchor: Option<WindowAnchor>,
}

impl Default for SimulaDiagnosticsSettings {
    fn default() -> Self {
        SimulaDiagnosticsSettings {
            config_path: "data/settings.diagnostics.ron".to_string(),
            collapsible: true,
            vscroll: true,
            resizable: true,
            anchor: None,
        }
    }
}

pub struct SimulaDiagnosticsPlugin;

impl Plugin for SimulaDiagnosticsPlugin {
    fn build(&self, app: &mut App) {
        app.world
            .get_resource_or_insert_with(SimulaDiagnosticsSettings::default);

        app.add_plugin(FrameTimeDiagnosticsPlugin::default())
            .insert_resource(DiagnosticParameters { framerate: 0.00 })
            .add_plugin(DiagnosticsLoaderPlugin)
            .add_system(ui_system_diagnostics)
            .add_system(framerate_update_system);
    }
}
