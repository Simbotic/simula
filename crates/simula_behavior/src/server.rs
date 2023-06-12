use crate::{
    prelude::*,
    protocol::{
        BehaviorFileId, BehaviorFileName, BehaviorProtocolClient, BehaviorProtocolServer,
        BehaviorServer, BehaviorState, BehaviorTelemetry,
    },
};
use bevy::{prelude::*, utils::HashMap};
use serde::Serialize;

#[derive(Default)]
pub struct BehaviorServerPlugin<T: BehaviorFactory>(pub std::marker::PhantomData<T>);

impl<T> Plugin for BehaviorServerPlugin<T>
where
    T: BehaviorFactory + Serialize,
{
    fn build(&self, app: &mut App) {
        app.insert_resource(BehaviorTrackers::<T>::default())
            .add_startup_system(setup::<T>)
            .add_system(track_loaded_behaviors::<T>)
            .add_system(update::<T>)
            .add_system(update_telemetry::<T>);
    }
}

#[derive(Clone)]
pub struct BehaviorTracker<T: BehaviorFactory> {
    pub file_name: BehaviorFileName,
    pub entity: Option<Entity>,
    pub telemetry: bool,
    pub asset: Option<Handle<BehaviorAsset<T>>>,
}

#[derive(Default, Resource, Deref, DerefMut)]
pub struct BehaviorTrackers<T: BehaviorFactory>(HashMap<BehaviorFileId, BehaviorTracker<T>>);

fn track_loaded_behaviors<T: BehaviorFactory>(
    mut asset_events: EventReader<AssetEvent<BehaviorAsset<T>>>,
    asset_server: Res<AssetServer>,
    mut behavior_trackers: ResMut<BehaviorTrackers<T>>,
    behavior_server: Res<BehaviorServer<T>>,
    behavior_assets: Res<Assets<BehaviorAsset<T>>>,
) {
    for event in asset_events.iter() {
        match event {
            AssetEvent::Created { handle } => {
                if let Some(path) = asset_server.get_handle_path(handle) {
                    println!("Created: {:?}", path);

                    // check if there is a tracker for this asset
                    let behavior_tracker = behavior_trackers.iter_mut().find(|(_, tracker)| {
                        if let Some(asset) = &tracker.asset {
                            asset == handle
                        } else {
                            false
                        }
                    });

                    // if there is a tracker send asset to clients
                    if let Some((file_id, _behavior_tracker)) = behavior_tracker {
                        if let Some(asset) = behavior_assets.get(handle) {
                            // Server send asset to clients
                            behavior_server
                                .sender
                                .send(BehaviorProtocolServer::File(
                                    file_id.clone(),
                                    asset.behavior.clone(),
                                ))
                                .unwrap();
                        }
                    }
                    // if there is no tracker, create one, notify file name to clients
                    else {
                        // Get asset path and build a tracker for it
                        let file_path = path.path().to_string_lossy();
                        let file_name = file_path.trim_end_matches(".bht.ron").to_owned();

                        let behavior_file_id = BehaviorFileId::new();
                        let behavior_file_name = BehaviorFileName(file_name.into());

                        behavior_trackers.insert(
                            behavior_file_id.clone(),
                            BehaviorTracker {
                                file_name: behavior_file_name.clone(),
                                entity: None,
                                telemetry: false,
                                asset: Some(handle.clone()),
                            },
                        );

                        // Server send file name to clients
                        behavior_server
                            .sender
                            .send(BehaviorProtocolServer::FileName(
                                behavior_file_id,
                                behavior_file_name,
                            ))
                            .unwrap();
                    }
                } else {
                    error!("Asset has no soource path: {:?}", event);
                }
            }
            _ => error!("TODO: Unhandled asset event: {:?}", event),
        }
    }
}

fn setup<T: BehaviorFactory>(
    mut behavior_trackers: ResMut<BehaviorTrackers<T>>,
    behavior_server: Res<BehaviorServer<T>>,
) {
    let dir_path = "assets/bhts/u";

    // Read the directory and handle any errors
    let paths = match std::fs::read_dir(dir_path) {
        Ok(paths) => paths,
        Err(err) => {
            eprintln!("Error reading directory: {}", err);
            return;
        }
    };

    // Iterate over the directory entries
    for path in paths {
        if let Ok(entry) = path {
            // Check if the entry is a file with the desired extension
            if entry.file_type().unwrap().is_file() {
                let osfile_name = entry.file_name();
                let file_name = osfile_name.to_string_lossy().to_owned();
                if file_name.ends_with(".bht.ron") {
                    let file_id = BehaviorFileId::new();
                    let file_name = format!("bhts/u/{}", file_name.trim_end_matches(".bht.ron"));
                    let file_name = BehaviorFileName(file_name.to_string().into());

                    behavior_trackers.insert(
                        file_id.clone(),
                        BehaviorTracker {
                            file_name: file_name.clone(),
                            entity: None,
                            telemetry: false,
                            asset: None,
                        },
                    );

                    behavior_server
                        .sender
                        .send(BehaviorProtocolServer::FileName(file_id, file_name))
                        .unwrap();
                }
            }
        }
    }
}

fn build_telemetry<T: BehaviorFactory>(
    world: &mut World,
    entity: Entity,
    telemetry: &mut BehaviorTelemetry<T>,
    behavior: &Behavior<T>,
) -> Result<(), BehaviorMissing> {
    let behavior_running = world.get::<BehaviorRunning>(entity);
    let behavior_failure = world.get::<BehaviorFailure>(entity);
    let behavior_success = world.get::<BehaviorSuccess>(entity);
    let behavior_cursor = world.get::<BehaviorCursor>(entity);
    let behavior_state = if behavior_cursor.is_some() {
        BehaviorState::Cursor
    } else if behavior_running.is_some() {
        BehaviorState::Running
    } else if behavior_failure.is_some() {
        BehaviorState::Failure
    } else if behavior_success.is_some() {
        BehaviorState::Success
    } else {
        BehaviorState::None
    };

    // Copy data from entity to telemetry
    let mut data = behavior.data().clone();
    data.copy_from(entity, world)?;

    let mut telemetry_children = vec![];

    let instance_children = world.get::<BehaviorChildren>(entity).cloned();
    let source_children = behavior.nodes().iter();
    if let Some(instance_children) = instance_children {
        let instance_children = instance_children.iter();
        for (instance_child, source_child) in instance_children.zip(source_children) {
            let mut telemetry = BehaviorTelemetry::<T>::default();
            build_telemetry(world, *instance_child, &mut telemetry, source_child)?;
            telemetry_children.push(telemetry);
        }
    }

    *telemetry = BehaviorTelemetry(behavior_state, Some(data), telemetry_children);

    Ok(())
}

fn update_telemetry<T: BehaviorFactory>(world: &mut World) {
    let mut tracks = vec![];
    if let Some(behavior_trackers) = world.get_resource::<BehaviorTrackers<T>>() {
        for (file_id, behavior_tracker) in behavior_trackers.iter() {
            if behavior_tracker.telemetry {
                if let Some(behavior_asset) = &behavior_tracker.asset {
                    if let Some(behavior_assets) = world.get_resource::<Assets<BehaviorAsset<T>>>()
                    {
                        if let Some(behavior_asset) = behavior_assets.get(&behavior_asset) {
                            if let Some(entity) = behavior_tracker.entity {
                                tracks.push((
                                    file_id.clone(),
                                    entity,
                                    behavior_asset.behavior.clone(),
                                ));
                            } else {
                                error!("Behavior has no entity");
                            }
                        }
                    } else {
                        error!("Failed to get behavior assets");
                    }
                } else {
                    error!("Behavior has no asset");
                }
            }
        }
    } else {
        error!("Failed to get behavior trackers");
    }

    for (file_id, entity, behavior) in tracks {
        if let Some(behavior_tree) = world.get::<BehaviorTree<T>>(entity) {
            if let Some(root) = behavior_tree.root {
                let mut telemetry = BehaviorTelemetry::<T>::default();
                if build_telemetry(world, root, &mut telemetry, &behavior).is_ok() {
                    let behavior_server = world.get_resource::<BehaviorServer<T>>().unwrap();
                    behavior_server
                        .sender
                        .send(BehaviorProtocolServer::Telemetry(file_id, telemetry))
                        .unwrap();
                } else {
                    error!("Failed to build telemetry");
                }
            }
        }
    }
}

fn update<T: BehaviorFactory>(
    mut commands: Commands,
    behavior_trees: Query<&BehaviorTree<T>>,
    mut behavior_assets: ResMut<Assets<BehaviorAsset<T>>>,
    mut behavior_trackers: ResMut<BehaviorTrackers<T>>,
    behavior_server: Res<BehaviorServer<T>>,
    asset_server: Res<AssetServer>,
) where
    T: Serialize,
{
    while let Ok(client_msg) = &behavior_server.receiver.try_recv() {
        match client_msg {
            BehaviorProtocolClient::LoadFile(file_id) => {
                if let Some(behavior_tracker) = behavior_trackers.get_mut(&file_id) {
                    // check if behavior is already loaded
                    if let Some(behavior_asset) = &behavior_tracker.asset {
                        if let Some(behavior_asset) = behavior_assets.get(&behavior_asset) {
                            behavior_server
                                .sender
                                .send(BehaviorProtocolServer::File(
                                    file_id.clone(),
                                    behavior_asset.behavior.clone(),
                                ))
                                .unwrap();
                        }
                    }
                    // if not loaded, load and get a handle to asset
                    else {
                        let behavior_handle: Handle<BehaviorAsset<T>> = asset_server
                            .load(format!("{}.bht.ron", *behavior_tracker.file_name).as_str());
                        behavior_tracker.asset = Some(behavior_handle);
                    }
                } else {
                    error!("Invalid file_id: {:?}", file_id);
                }
            }
            BehaviorProtocolClient::SaveFile(file_id, file_name, file_data) => {
                // a tracker is required to run a behavior
                if let Some(behavior_tracker) = behavior_trackers.get_mut(&file_id) {
                    let file_data =
                        ron::ser::to_string_pretty(&file_data, ron::ser::PrettyConfig::default());
                    match file_data {
                        Ok(file_data) => {
                            behavior_tracker.file_name = file_name.clone();
                            let dir_path = "assets";
                            let file_ext = "bht.ron";
                            let file_path = format!("{}/{}.{}", dir_path, **file_name, file_ext);
                            std::fs::write(&file_path, file_data).unwrap();
                            info!("Saved file: {}", &file_path);
                            behavior_server
                                .sender
                                .send(BehaviorProtocolServer::FileSaved(file_id.clone()))
                                .unwrap();
                        }
                        Err(err) => {
                            error!("Failed to serialize file_data: {:?}", err);
                        }
                    }
                } else {
                    error!("Invalid file_id: {:?}", file_id);
                }
            }
            BehaviorProtocolClient::Run(file_id, behavior) => {
                // a tracker is required to run a behavior
                if let Some(behavior_tracker) = behavior_trackers.get_mut(&file_id) {
                    // if the behavior is already loaded, update it
                    if let Some(behavior_asset) = &behavior_tracker.asset {
                        if let Some(asset) = behavior_assets.get_mut(&behavior_asset) {
                            // update behavior in asset resource
                            asset.behavior = behavior.clone();

                            // delete any existing behavior tree entity, and create a new one
                            let behavior_tree_entity = {
                                if let Some(behavior_tree_entity) = behavior_tracker.entity {
                                    if let Ok(behavior_tree) =
                                        behavior_trees.get(behavior_tree_entity)
                                    {
                                        if let Some(root) = behavior_tree.root {
                                            commands.entity(root).despawn_recursive();
                                        }
                                    }
                                    behavior_tree_entity
                                } else {
                                    let behavior_tree_entity = commands
                                        .spawn(Name::new(format!(
                                            "BHT: {}",
                                            *behavior_tracker.file_name
                                        )))
                                        .id();
                                    behavior_tracker.entity = Some(behavior_tree_entity);
                                    behavior_tree_entity
                                }
                            };

                            BehaviorTree::<T>::build_tree(
                                behavior_tree_entity,
                                &mut commands,
                                &behavior,
                            );

                            behavior_server
                                .sender
                                .send(BehaviorProtocolServer::Started(file_id.clone()))
                                .unwrap();
                        } else {
                            error!(
                                "Failed to get behavior asset resource for file_id: {:?}",
                                file_id
                            );
                        }
                    } else {
                        error!("Behavior is not loaded for file_id: {:?}", file_id);
                    }
                } else {
                    error!("Invalid file_id: {:?}", file_id);
                }
            }
            BehaviorProtocolClient::Stop(file_id) => {
                if let Some(behavior_tracker) = behavior_trackers.get_mut(&file_id) {
                    behavior_tracker.telemetry = false;

                    if let Some(behavior_tree_entity) = behavior_tracker.entity {
                        if let Ok(behavior_tree) = behavior_trees.get(behavior_tree_entity) {
                            if let Some(root) = behavior_tree.root {
                                commands.entity(root).despawn_recursive();
                            }
                        }
                        commands.entity(behavior_tree_entity).despawn_recursive();
                    }
                    behavior_tracker.entity = None;
                    behavior_server
                        .sender
                        .send(BehaviorProtocolServer::Stopped(file_id.clone()))
                        .unwrap();
                } else {
                    error!("Invalid file_id: {:?}", file_id);
                }
            }
            BehaviorProtocolClient::Telemetry(file_id, enable) => {
                if let Some(behavior_tracker) = behavior_trackers.get_mut(&file_id) {
                    behavior_tracker.telemetry = *enable;
                } else {
                    error!("Invalid file_id: {:?}", file_id);
                }
            }
        }
    }
}
