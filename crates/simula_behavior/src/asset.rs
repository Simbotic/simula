use crate::{BehaviorSpawner, BehaviorTree};
use bevy::{
    asset::{AssetLoader, LoadContext, LoadedAsset},
    prelude::*,
    reflect::TypeUuid,
    utils::BoxedFuture,
};
use serde::{Deserialize, Serialize};
use std::fmt::Debug;

#[derive(Default, Debug, Serialize, Deserialize)]
pub struct BTNode<T: Default>(pub T, pub Vec<BTNode<T>>);

#[derive(Default, Debug, Serialize, Deserialize)]
pub struct BehaviorDocument<T: Default> {
    pub root: BTNode<T>,
}

#[derive(Default, Debug, TypeUuid, Deserialize)]
#[uuid = "7f117190-5353-11ed-ae42-02a179e5df2b"]
pub struct BehaviorAsset<T>
where
    T: Default + Debug,
{
    pub path: String,
    pub document: BehaviorDocument<T>,
}

#[derive(Default)]
pub struct BehaviorAssetLoader<T>(std::marker::PhantomData<T>);

impl<T> AssetLoader for BehaviorAssetLoader<T>
where
    T: Sync + Send + 'static + Default + Debug + TypeUuid + for<'de> Deserialize<'de>,
{
    fn load<'a>(
        &'a self,
        bytes: &'a [u8],
        load_context: &'a mut LoadContext,
    ) -> BoxedFuture<'a, Result<(), bevy::asset::Error>> {
        Box::pin(async move {
            let document: BehaviorDocument<T> = ron::de::from_bytes(&bytes)?;
            let asset = BehaviorAsset::<T> {
                path: load_context.path().display().to_string(),
                document,
            };
            println!("WWWWWWWWWW   Loaded asset: {:?}", asset);
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
    pub document: Handle<BehaviorAsset<T>>,
    pub parent: Option<Entity>,
}

pub fn async_loader<T>(
    mut commands: Commands,
    loaded_assets: Res<Assets<BehaviorAsset<T>>>,
    queued_assets: Query<(Entity, &BehaviorAssetLoading<T>)>,
) where
    T: BehaviorSpawner + TypeUuid + Send + Sync + 'static + Default + Debug,
{
    for (entity, queued_asset) in queued_assets.iter() {
        if let Some(loaded_asset) = loaded_assets.get(&queued_asset.document) {
            let BehaviorAsset { document, .. } = loaded_asset;
            let BehaviorDocument { root } = document;
            BehaviorTree::insert_tree::<T>(entity, queued_asset.parent, &mut commands, &root);
            commands.entity(entity).remove::<BehaviorAssetLoading<T>>();
        }
    }
}
