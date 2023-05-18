use asset::{RhaiScript, RhaiScriptLoader};
use bevy::prelude::*;
use bevy_console::{
    reply, AddConsoleCommand, ConsoleCommand, ConsoleCommandEntered, ConsoleConfiguration,
    ConsolePlugin, ConsoleSet,
};
use clap::Parser;
use rhai::{Dynamic, Engine, RegisterFn};

mod asset;

/// Evaluate a Rhai expression
#[derive(Parser, ConsoleCommand)]
#[command(name = "=")]
struct RhaiCommand {
    /// Expression to evaluate
    pub expr: Vec<String>,
}

pub struct ScriptPlugin;

impl Plugin for ScriptPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugin(ConsolePlugin)
            .add_asset::<RhaiScript>()
            .init_asset_loader::<RhaiScriptLoader>()
            .insert_resource(ConsoleConfiguration {
                ..Default::default()
            })
            .add_console_command::<RhaiCommand, _>(|mut cmd: ConsoleCommand<RhaiCommand>| {
                if let Some(Ok(RhaiCommand { expr })) = cmd.take() {
                    let mut engine = Engine::new();
                    engine.register_fn("add", |x: i64, y: i64| x + y);

                    let expr = expr.join(" ");
                    match engine.eval::<Dynamic>(&expr) {
                        Ok(result) => reply!(cmd, "{}", result),
                        Err(e) => reply!(cmd, "Error: {}", e),
                    }
                }
            })
            .add_system(raw_commands.in_set(ConsoleSet::Commands))
            .add_system(script_changed)
            .add_startup_system(load_script);
    }
}

fn raw_commands(mut console_commands: EventReader<ConsoleCommandEntered>) {
    for ConsoleCommandEntered { command_name, args } in console_commands.iter() {
        debug!(r#"Entered command "{command_name}" with args {:#?}"#, args);
    }
}

fn load_script(asset_server: Res<AssetServer>, mut commands: Commands, _scripts: Res<Assets<RhaiScript>>) {
    let script_handle: Handle<RhaiScript> = asset_server.load("scripts/behaviors.rhai");
    commands.spawn(script_handle);
    // if let Some(script) = scripts.get(&script_handle) {
    //     println!("Script {:?} was loaded", script_handle.id());
    // }
}

fn script_changed(mut script_events: EventReader<AssetEvent<RhaiScript>>) {
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
