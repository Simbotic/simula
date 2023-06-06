use asset::ScriptLoader;
pub use asset::{Scope, Script};
use bevy::prelude::*;
pub use rhai as script;

mod asset;

pub struct ScriptPlugin;

impl Plugin for ScriptPlugin {
    fn build(&self, app: &mut App) {
        app.add_asset::<Script>()
            .add_asset::<Scope>()
            .init_asset_loader::<ScriptLoader>()
            .add_system(script_changed);
    }
}

fn script_changed(mut script_events: EventReader<AssetEvent<Script>>) {
    for event in script_events.iter() {
        match event {
            AssetEvent::Created { handle } => {
                println!("Script {:?} was created", handle.id());
            }
            AssetEvent::Modified { handle } => {
                println!("Script {:?} was modified", handle.id());
            }
            AssetEvent::Removed { handle } => {
                println!("Script {:?} was removed", handle.id());
            }
        }
    }
}
