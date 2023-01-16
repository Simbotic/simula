use bevy::prelude::*;
use simula_behavior::prelude::*;

use self::{cop::CopBehaviorPlugin, robber::RobberBehaviorPlugin};

pub mod cop;
pub mod robber;

pub struct MissionBehaviorPlugin;

impl Plugin for MissionBehaviorPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugin(BehaviorPlugin)
            .add_plugin(BehaviorInspectorPlugin)
            .add_plugin(CopBehaviorPlugin)
            .add_plugin(RobberBehaviorPlugin);
    }
}
