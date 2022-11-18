use crate::asset_info::AssetInfo;
use bevy::prelude::*;
#[derive(Default, Component)]
pub struct Agent;

#[derive(Debug, Default, Component, Clone)]
pub struct AgentProductionType<T: AssetInfo>(pub T);

#[derive(Debug, Default, Component, Clone)]
pub struct AgentPurchaseType<T: AssetInfo>(pub T);

// pub trait AgentInfo<T: AssetInfo>: Component + Default {
//     type AgentAttributes: Component + Clone + Default;

//     fn production_type(&self, agent: &Self::AgentAttributes) -> Option<T>;
//     fn purchase_type(&self, attributes: &Self::AgentAttributes) -> Option<T>;
// }




// impl AgentInfo for Agent{
//     // type AgentAttributes: Component + Clone + Default;

//     fn production_type(&self) -> Option<T> {T}
//     fn purchase_type(&self) -> Option<T>{T}
// }

// pub struct AgentAttributes {}
