use bevy::prelude::*;
use simula_behavior::prelude::*;

pub mod bank;
pub mod cop;
pub mod robber;

pub struct MissionBehaviorPlugin;

impl Plugin for MissionBehaviorPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugin(BehaviorPlugin)
            .add_plugin(BehaviorInspectorPlugin)
            .add_plugin(bank::BankBehaviorPlugin)
            .add_plugin(cop::CopBehaviorPlugin)
            .add_plugin(robber::RobberBehaviorPlugin);
    }
}
