use crate::{BehaviorFactory, BehaviorTree};
use bevy::{
    asset::{AssetLoader, LoadContext, LoadedAsset},
    prelude::*,
    reflect::TypeUuid,
    utils::BoxedFuture,
};
use serde::{Deserialize, Serialize};
use std::fmt::Debug;

#[derive(Default, Debug, Serialize, Deserialize, Clone)]
pub struct Behavior<T: Default>(pub String, pub T, #[serde(default)] pub Vec<Behavior<T>>);

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
    pub node: Handle<BehaviorAsset<T>>,
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
        if let Some(loaded_asset) = loaded_assets.get(&queued_asset.node) {
            BehaviorTree::insert_tree::<T>(
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
