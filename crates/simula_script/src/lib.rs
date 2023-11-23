use asset::ScriptLoader;
pub use asset::{Script, ScriptContext};
use bevy::prelude::*;
pub use rhai as script;

mod asset;

pub struct ScriptPlugin;

impl Plugin for ScriptPlugin {
    fn build(&self, app: &mut App) {
        app.init_asset::<Script>()
            .init_asset::<ScriptContext>()
            .init_asset_loader::<ScriptLoader>()
            .add_systems(Update, script_changed);
    }
}

fn script_changed(mut script_events: EventReader<AssetEvent<Script>>) {
    for event in script_events.read() {
        match event {
            AssetEvent::Added { id } => {
                info!("Script {:?} was created", id);
            }
            AssetEvent::Modified { id } => {
                info!("Script {:?} was modified", id);
            }
            AssetEvent::Removed { id } => {
                info!("Script {:?} was removed", id);
            }
            AssetEvent::LoadedWithDependencies { id } => {
                info!("Script {:?} was loaded with dependencies", id);
            }
        }
    }
}
