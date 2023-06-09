use crate::{BehaviorFactory, BehaviorTree};
use bevy::{
    asset::{AssetLoader, LoadContext, LoadedAsset},
    prelude::*,
    reflect::TypeUuid,
    utils::BoxedFuture,
};
use serde::{Deserialize, Serialize};
use std::borrow::Cow;
use std::fmt::Debug;

#[derive(Default, Debug, Serialize, Deserialize, Clone)]
pub struct BehaviorAttributes {
    pub pos: Vec2,
}

#[derive(Default, Debug, Serialize, Deserialize, Clone)]
pub struct Behavior<T: Default>(
    Cow<'static, str>,
    T,
    #[serde(default)] Vec<Behavior<T>>,
    #[serde(default)] BehaviorAttributes,
);

impl<T> Behavior<T>
where
    T: Default,
{
    pub fn new(
        name: impl Into<Cow<'static, str>>,
        data: T,
        attrs: BehaviorAttributes,
        nodes: Vec<Behavior<T>>,
    ) -> Self {
        Self(name.into(), data, nodes, attrs)
    }

    pub fn name(&self) -> &str {
        &self.0
    }

    pub fn data(&self) -> &T {
        &self.1
    }

    pub fn attrs(&self) -> &BehaviorAttributes {
        &self.3
    }

    pub fn nodes(&self) -> &Vec<Behavior<T>> {
        &self.2
    }

    pub fn nodes_mut(&mut self) -> &mut Vec<Behavior<T>> {
        &mut self.2
    }
}

#[derive(Default, Debug, TypeUuid, Deserialize)]
#[uuid = "7f117190-5353-11ed-ae42-02a179e5df2b"]
pub struct BehaviorAsset<T>
where
    T: TypeUuid + Default,
{
    pub path: String,
    pub behavior: Behavior<T>,
}

#[derive(Default)]
pub struct BehaviorAssetLoader<T>(std::marker::PhantomData<T>);

impl<T> AssetLoader for BehaviorAssetLoader<T>
where
    T: Default + TypeUuid + Send + Sync + 'static + for<'de> Deserialize<'de>,
{
    fn load<'a>(
        &'a self,
        bytes: &'a [u8],
        load_context: &'a mut LoadContext,
    ) -> BoxedFuture<'a, Result<(), bevy::asset::Error>> {
        Box::pin(async move {
            let document = std::str::from_utf8(bytes)?.to_string();
            let behavior = ron::de::from_str(&document)?;
            let asset = BehaviorAsset::<T> {
                path: load_context.path().display().to_string(),
                behavior,
            };
            load_context.set_default_asset(LoadedAsset::new(asset));
            Ok(())
        })
    }

    fn extensions(&self) -> &[&str] {
        &["bht.ron"]
    }
}

#[derive(Component)]
pub struct BehaviorAssetLoading<T>
where
    T: TypeUuid + Send + Sync + 'static + Default + Debug,
{
    pub asset: Handle<BehaviorAsset<T>>,
    pub tree: Entity,
    pub parent: Option<Entity>,
    pub phantom: std::marker::PhantomData<T>,
}

pub fn behavior_loader<T>(
    mut commands: Commands,
    loaded_assets: Res<Assets<BehaviorAsset<T>>>,
    queued_assets: Query<(Entity, &BehaviorAssetLoading<T>)>,
) where
    T: BehaviorFactory + for<'de> Deserialize<'de>,
{
    for (entity, queued_asset) in queued_assets.iter() {
        if let Some(loaded_asset) = loaded_assets.get(&queued_asset.asset) {
            BehaviorTree::insert_tree(
                queued_asset.tree,
                entity,
                queued_asset.parent,
                &mut commands,
                &loaded_asset.behavior,
            );
            commands.entity(entity).remove::<BehaviorAssetLoading<T>>();
        }
    }
}
