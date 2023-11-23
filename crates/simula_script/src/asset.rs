use bevy::{
    asset::{io::Reader, AssetLoader, AsyncReadExt, LoadContext},
    prelude::*,
    reflect::TypeUuid,
    utils::BoxedFuture,
};
use rhai as script;
use serde::Deserialize;
use std::borrow::Cow;
use thiserror::Error;

#[derive(Asset, TypePath, TypeUuid)]
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

#[derive(Asset, TypePath, Default, Debug, TypeUuid, Deserialize)]
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

#[non_exhaustive]
#[derive(Debug, Error)]
pub enum ScriptLoaderError {
    /// An [IO](std::io) Error
    #[error("Could not load asset: {0}")]
    Io(#[from] std::io::Error),
    // An [UTF8](std::str::Utf8Error) Error
    #[error("Could not parse asset as UTF8: {0}")]
    Utf8(#[from] std::string::FromUtf8Error),
}

impl AssetLoader for ScriptLoader {
    type Asset = Script;
    type Settings = ();
    type Error = ScriptLoaderError;
    fn load<'a>(
        &'a self,
        reader: &'a mut Reader,
        _settings: &'a (),
        _load_context: &'a mut LoadContext,
    ) -> BoxedFuture<'a, Result<Self::Asset, Self::Error>> {
        Box::pin(async move {
            let mut bytes = Vec::new();
            reader.read_to_end(&mut bytes).await?;
            let script = String::from_utf8(bytes.to_vec())?;
            Ok(Script {
                script: script.into(),
                ast: None,
            })
        })
    }

    fn extensions(&self) -> &[&str] {
        static EXTENSIONS: &[&str] = &["rhai"];
        EXTENSIONS
    }
}
