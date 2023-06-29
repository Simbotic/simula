use asset::ScriptLoader;
pub use asset::{Script, ScriptContext};
use bevy::prelude::*;
pub use rhai as script;

mod asset;

pub struct ScriptPlugin;

impl Plugin for ScriptPlugin {
    fn build(&self, app: &mut App) {
        app.add_asset::<Script>()
            .add_asset::<ScriptContext>()
            .init_asset_loader::<ScriptLoader>()
            .add_system(script_changed);
    }
}

fn script_changed(mut script_events: EventReader<AssetEvent<Script>>) {
    for event in script_events.iter() {
        match event {
            AssetEvent::Created { handle } => {
                info!("Script {:?} was created", handle.id());
            }
            AssetEvent::Modified { handle } => {
                info!("Script {:?} was modified", handle.id());
            }
            AssetEvent::Removed { handle } => {
                info!("Script {:?} was removed", handle.id());
            }
        }
    }
}
