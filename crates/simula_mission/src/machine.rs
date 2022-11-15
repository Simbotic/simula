use bevy::prelude::*;
use crate::asset_ui::AssetInfo;

#[derive(Debug, Default, Component, Reflect, Clone)]
pub struct Machine;

#[derive(Debug, Default, Component, Clone)]
pub struct MachineType<T: Component + AssetInfo>(pub T);