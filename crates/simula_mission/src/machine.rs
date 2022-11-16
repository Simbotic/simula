use crate::asset_info::AssetInfo;
use bevy::prelude::*;

#[derive(Debug, Default, Component, Reflect, Clone)]
pub struct Machine;

#[derive(Debug, Default, Component, Clone)]
pub struct MachineType<T: Component + AssetInfo>(pub T);
