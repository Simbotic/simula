use crate::asset_info::AssetInfo;
use bevy::prelude::*;
#[derive(Default, Component)]
pub struct Agent;

#[derive(Debug, Default, Component, Clone)]
pub struct AgentProductionType<T: AssetInfo>(pub T);

#[derive(Debug, Default, Component, Clone)]
pub struct AgentPurchaseType<T: AssetInfo>(pub T);
