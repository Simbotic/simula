use bevy::{
    asset::{AssetLoader, LoadContext, LoadedAsset},
    prelude::*,
    reflect::TypeUuid,
    utils::BoxedFuture,
};
use rhai as script;
use serde::Deserialize;

#[derive(TypeUuid)]
#[uuid = "1EDAA495-674E-45AA-903B-212D088BD991"]
pub struct Scope {
    pub engine: script::Engine,
    pub scope: script::Scope<'static>,
}

impl Scope {
    pub fn new() -> Self {
        let mut engine = script::Engine::new();
        engine.on_print(|x| info!("{x}"));
        let scope = script::Scope::new();
        Self { engine, scope }
    }
}

#[derive(Default, Debug, TypeUuid, Deserialize)]
#[uuid = "6687C58B-CCE2-4BD2-AD28-7AA3ED6C355B"]
pub struct Script {
    pub script: String,
    #[serde(skip)]
    ast: script::AST,
}

impl Script {
    pub fn from_str(script: &str) -> Result<Self, Box<script::ParseError>> {
        let engine = script::Engine::new();
        let ast = engine.compile(&script)?;
        Ok(Script {
            script: script.into(),
            ast,
        })
    }

    pub fn eval<T>(&self, context: &mut Scope) -> Result<T, std::boxed::Box<script::EvalAltResult>>
    where
        T: Clone + Deserialize<'static> + Send + Sync + 'static,
    {
        let stack = context.scope.len();
        let result = context
            .engine
            .eval_ast_with_scope::<T>(&mut context.scope, &self.ast);
        context.scope.rewind(stack);
        result
    }
}

#[derive(Default)]
pub struct ScriptLoader;

impl AssetLoader for ScriptLoader {
    fn load<'a>(
        &'a self,
        bytes: &'a [u8],
        load_context: &'a mut LoadContext,
    ) -> BoxedFuture<'a, Result<(), bevy::asset::Error>> {
        Box::pin(async move {
            let engine = script::Engine::new();
            let script = String::from_utf8(bytes.to_vec()).unwrap();
            let ast = engine.compile(&script)?;
            load_context.set_default_asset(LoadedAsset::new(Script { script, ast }));
            Ok(())
        })
    }

    fn extensions(&self) -> &[&str] {
        static EXTENSIONS: &[&str] = &["rhai"];
        EXTENSIONS
    }
}
