use bevy::{
    asset::{AssetLoader, LoadContext, LoadedAsset},
    prelude::*,
    reflect::TypeUuid,
    utils::BoxedFuture,
};
use rhai::{Engine, EvalAltResult, Map, ParseError, Scope, AST};
use serde::Deserialize;

#[derive(Resource)]
pub struct RhaiContext {
    pub engine: Engine,
    pub scope: Scope<'static>,
}

impl RhaiContext {
    pub fn new() -> Self {
        let mut engine = Engine::new();
        engine.on_print(|x| info!("{x}"));
        let mut scope = Scope::new();
        let mut global = Map::new();
        global.insert("state".into(), 0.into());
        scope.push("global", global);
        Self { engine, scope }
    }
}

#[derive(Default, Debug, TypeUuid, Deserialize)]
#[uuid = "6687C58B-CCE2-4BD2-AD28-7AA3ED6C355B"]
pub struct RhaiScript {
    pub script: String,
    #[serde(skip)]
    pub ast: AST,
}

impl RhaiScript {
    pub fn from_str(script: &str) -> Result<Self, Box<ParseError>> {
        let engine = Engine::new();
        let ast = engine.compile(&script)?;
        Ok(RhaiScript {
            script: script.into(),
            ast,
        })
    }

    pub fn eval<T>(&self, context: &mut RhaiContext) -> Result<T, std::boxed::Box<EvalAltResult>>
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
