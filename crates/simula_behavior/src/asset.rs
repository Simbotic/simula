use crate::{BehaviorChildren, BehaviorCursor, BehaviorFactory, BehaviorNode, BehaviorTree};
use bevy::{
    asset::{AssetLoader, LoadContext, LoadedAsset},
    prelude::*,
    reflect::TypeUuid,
    utils::BoxedFuture,
};
use serde::{Deserialize, Serialize};
use std::borrow::Cow;
use std::fmt::Debug;

/// This is the one and only data type for creating behaviors.
/// The idea is to have an extremely simple data type that can be serialized,
/// deserialized, generated from script, code or AI.
#[derive(Default, Debug, Serialize, Deserialize, Clone)]
pub struct Behavior<T: BehaviorFactory>(
    Cow<'static, str>,
    T,
    #[serde(default)] Vec<Behavior<T>>,
    #[serde(default)] T::Attributes,
);

impl<T> Behavior<T>
where
    T: Default + BehaviorFactory,
{
    pub fn new(
        name: impl Into<Cow<'static, str>>,
        data: T,
        attrs: T::Attributes,
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

    pub fn attrs(&self) -> &T::Attributes {
        &self.3
    }

    pub fn attrs_mut(&mut self) -> &mut T::Attributes {
        &mut self.3
    }

    pub fn nodes(&self) -> &Vec<Behavior<T>> {
        &self.2
    }

    pub fn nodes_mut(&mut self) -> &mut Vec<Behavior<T>> {
        &mut self.2
    }
}

#[derive(Default, Debug, TypeUuid, Deserialize)]
#[uuid = "B543FD10-86EA-42A6-BC87-2A9DB57BFBAD"]

pub struct BehaviorAsset<T>
where
    T: BehaviorFactory,
{
    pub behavior: Behavior<T>,
    pub file_name: Option<Cow<'static, str>>,
}

#[derive(Default, Debug, TypeUuid, Deserialize, Deref)]
#[uuid = "7f117190-5353-11ed-ae42-02a179e5df2b"]
pub struct BehaviorDocument(String);

#[derive(Default)]
pub struct BehaviorAssetLoader;

impl AssetLoader for BehaviorAssetLoader {
    fn load<'a>(
        &'a self,
        bytes: &'a [u8],
        load_context: &'a mut LoadContext,
    ) -> BoxedFuture<'a, Result<(), bevy::asset::Error>> {
        Box::pin(async move {
            let document = std::str::from_utf8(bytes)?.to_string();
            let asset = BehaviorDocument(document);
            load_context.set_default_asset(LoadedAsset::new(asset));
            Ok(())
        })
    }

    fn extensions(&self) -> &[&str] {
        &["bht.ron"]
    }
}

#[derive(Component, Default)]
pub struct BehaviorTreeReset<T>
where
    T: TypeUuid + Send + Sync + 'static + Default + Debug + BehaviorFactory,
{
    pub phantom: std::marker::PhantomData<T>,
}

pub fn behavior_tree_reset<T>(
    mut commands: Commands,
    behavior_assets: Res<Assets<BehaviorAsset<T>>>,
    loadings: Query<
        (Entity, &Handle<BehaviorAsset<T>>, Option<&BehaviorNode>),
        (With<BehaviorTree<T>>, With<BehaviorTreeReset<T>>),
    >,
) where
    T: BehaviorFactory + for<'de> Deserialize<'de>,
{
    for (entity, behavior_asset, behavior_node) in loadings.iter() {
        // Remove behavior tree child nodes
        commands.entity(entity).clear_children();

        // Once asset is loaded, insert the tree nodes
        if let Some(behavior_asset) = behavior_assets.get(behavior_asset) {
            info!("Loading behavior tree for entity {:?}", entity);
            commands.entity(entity).remove::<BehaviorTreeReset<T>>();

            // if this behavior tree is a behavior node, then it will be a parent for the loaded tree root
            // this is for linking the trees together
            let parent = if behavior_node.is_some() {
                Some(entity)
            } else {
                None
            };

            // Build tree on root
            let root =
                BehaviorTree::insert_tree(entity, parent, &mut commands, &behavior_asset.behavior);

            // all tree nodes are linked, but tree root needs to be linked to parent
            commands.entity(entity).add_child(root);
            if behavior_node.is_some() {
                commands.entity(entity).insert(BehaviorChildren(vec![root]));
            }

            // start running behavior
            commands.entity(root).insert(BehaviorCursor::Delegate);
        }
    }
}

pub fn behavior_document_to_asset<T>(
    mut commands: Commands,
    behavior_documents: Res<Assets<BehaviorDocument>>,
    mut behavior_assets: ResMut<Assets<BehaviorAsset<T>>>,
    asset_server: Res<AssetServer>,
    documents: Query<(Entity, &Handle<BehaviorDocument>), With<BehaviorTree<T>>>,
) where
    T: BehaviorFactory + for<'de> Deserialize<'de>,
{
    for (entity, behavior_document_handle) in documents.iter() {
        // Convert document into behavior asset
        if let Some(behavior_document) = behavior_documents.get(behavior_document_handle) {
            // Remove document handle, if it fails to deserialize we wont keep trying
            commands.entity(entity).remove::<Handle<BehaviorDocument>>();
            // Deserialize behavior asset
            let res = ron::de::from_str::<Behavior<T>>(&behavior_document);
            if let Ok(behavior) = res {
                // Get file name
                let path = asset_server.get_handle_path(behavior_document_handle);
                let file_name = path.and_then(|path| {
                    let file_path = path.path().to_string_lossy();
                    let file_name: Cow<'static, str> =
                        file_path.trim_end_matches(".bht.ron").to_owned().into();
                    Some(file_name)
                });

                // Add behavior asset to asset manager
                let behavior_handle = behavior_assets.add(BehaviorAsset {
                    behavior,
                    file_name,
                });

                // and insert
                commands.entity(entity).insert(behavior_handle);
            } else if let Err(err) = res {
                error!(
                    "Failed to deserialize behavior tree for entity {:?} {}",
                    entity, err
                );
            }
        }
    }
}
