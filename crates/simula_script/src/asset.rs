use bevy::{
    asset::{AssetLoader, LoadContext, LoadedAsset},
    prelude::*,
    reflect::TypeUuid,
    utils::BoxedFuture,
};
use rhai as script;
use serde::Deserialize;
use std::borrow::Cow;

#[derive(TypeUuid)]
#[uuid = "1EDAA495-674E-45AA-903B-212D088BD991"]
pub struct ScriptContext {
    pub engine: script::Engine,
    pub scope: script::Scope<'static>,
}

impl ScriptContext {
    pub fn new() -> Self {
        let mut engine = script::Engine::new();
        engine.on_print(|x| info!("{x}"));
        let scope = script::Scope::new();
        Self { engine, scope }
    }

    pub fn eval<T>(&mut self, script: &str) -> Result<T, Box<script::EvalAltResult>>
    where
        T: Clone + Deserialize<'static> + Send + Sync + 'static,
    {
        let ast = self.engine.compile(&script)?;
        let stack = self.scope.len();
        let result = self.engine.eval_ast_with_scope::<T>(&mut self.scope, &ast);
        self.scope.rewind(stack);
        result
    }
}

// trait FromScript<InputType, OutputType> {
//     fn from_script(script: &str) -> Result<OutputType, Box<script::EvalAltResult>>;
// }

#[derive(Default, Debug, TypeUuid, Deserialize)]
#[uuid = "6687C58B-CCE2-4BD2-AD28-7AA3ED6C355B"]
pub struct Script {
    pub script: Cow<'static, str>,
    #[serde(skip)]
    ast: Option<script::AST>,
}

impl Script {
    pub fn compile(
        &mut self,
        context: &mut ScriptContext,
    ) -> Result<(), Box<script::EvalAltResult>> {
        let ast = context.engine.compile(&self.script)?;
        self.ast = Some(ast);
        Ok(())
    }

    pub fn eval<T>(
        &self,
        context: &mut ScriptContext,
    ) -> Result<T, std::boxed::Box<script::EvalAltResult>>
    where
        T: Clone + Send + Sync + 'static,
    {
        let ast = self.ast.as_ref().unwrap();
        let stack = context.scope.len();
        let result = context
            .engine
            .eval_ast_with_scope::<T>(&mut context.scope, ast);
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
            let script = String::from_utf8(bytes.to_vec()).unwrap();
            load_context.set_default_asset(LoadedAsset::new(Script {
                script: script.into(),
                ast: None,
            }));
            Ok(())
        })
    }

    fn extensions(&self) -> &[&str] {
        static EXTENSIONS: &[&str] = &["rhai"];
        EXTENSIONS
    }
}
