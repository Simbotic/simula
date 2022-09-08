use bevy::{
    asset::{AssetLoader, BoxedFuture, LoadContext, LoadedAsset},
    prelude::*,
    reflect::TypeUuid,
};
use serde::Deserialize;

use crate::diagnostics::{
    DiagnosticLimits, DiagnosticPowerPreference, DiagnosticWgpuSettingsPriority,
    SimulaDiagnosticsSettings,
};

#[derive(Debug, Deserialize, TypeUuid, PartialEq)]
#[uuid = "39cadc56-aa9c-4543-8640-a018b74b5052"]
pub struct RonDiagnostics {
    pub power_preference: DiagnosticPowerPreference,
    pub priority: DiagnosticWgpuSettingsPriority,
    pub limits: DiagnosticLimits,
}

#[derive(Default)]
pub struct DiagnosticsState {
    pub handle: Vec<HandleUntyped>,
    pub printed: bool,
}

#[derive(Default)]
pub struct DiagnosticsAssetLoader;

impl AssetLoader for DiagnosticsAssetLoader {
    fn load<'a>(
        &'a self,
        bytes: &'a [u8],
        load_context: &'a mut LoadContext,
    ) -> BoxedFuture<'a, Result<(), anyhow::Error>> {
        Box::pin(async move {
            let custom_asset = ron::de::from_bytes::<RonDiagnostics>(bytes)?;
            load_context.set_default_asset(LoadedAsset::new(custom_asset));
            Ok(())
        })
    }

    fn extensions(&self) -> &[&str] {
        &["diagnostics.ron"]
    }
}

pub fn load_diagnostics_config(
    diagnostics_settings: Res<SimulaDiagnosticsSettings>,
    mut collider_state: ResMut<DiagnosticsState>,
    asset_server: Res<AssetServer>,
) {
    collider_state
        .handle
        .push(asset_server.load_untyped(&diagnostics_settings.config_path.clone()));
}

pub struct DiagnosticsLoaderPlugin;

impl Plugin for DiagnosticsLoaderPlugin {
    fn build(&self, app: &mut App) {
        app.add_asset::<RonDiagnostics>()
            .init_asset_loader::<DiagnosticsAssetLoader>()
            .init_resource::<DiagnosticsState>()
            .add_startup_system(load_diagnostics_config);
    }
}
