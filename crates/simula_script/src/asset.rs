use bevy::{
    asset::{AssetLoader, LoadContext, LoadedAsset},
    prelude::*,
    reflect::TypeUuid,
    utils::BoxedFuture,
};
use rhai::{Engine, Scope, AST};
use serde::{Deserialize, Serialize};

#[derive(Default, Debug, TypeUuid, Deserialize, Component)]
#[uuid = "6687C58B-CCE2-4BD2-AD28-7AA3ED6C355B"]
pub struct RhaiScript {
    pub script: String,
    #[serde(skip)]
    pub ast: AST,
}

#[derive(Default)]
pub struct RhaiScriptLoader;

impl AssetLoader for RhaiScriptLoader {
    fn load<'a>(
        &'a self,
        bytes: &'a [u8],
        load_context: &'a mut LoadContext,
    ) -> BoxedFuture<'a, Result<(), bevy::asset::Error>> {
        Box::pin(async move {
            let engine = Engine::new();
            let script = String::from_utf8(bytes.to_vec()).unwrap();
            let ast = engine.compile(&script)?;
            load_context.set_default_asset(LoadedAsset::new(RhaiScript { script, ast }));
            Ok(())
        })
    }

    fn extensions(&self) -> &[&str] {
        static EXTENSIONS: &[&str] = &["rhai"];
        EXTENSIONS
    }
}
