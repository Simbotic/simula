use crate::{
    prelude::*,
    protocol::{
        BehaviorFileId, BehaviorFileName, BehaviorProtocolClient, BehaviorProtocolServer,
        BehaviorServer, BehaviorState, BehaviorTelemetry, StartOption, StopOption,
    },
};
use bevy::{prelude::*, utils::HashMap};
use serde::Serialize;
use std::{cmp::Ordering, collections::BinaryHeap, time::Duration};

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

#[derive(Clone, Debug)]
pub enum EntityTracker {
    None,
    Spawned(Entity),
    Attached(Entity),
    Inserted(Entity),
}

#[derive(Clone)]
pub struct BehaviorTracker<T: BehaviorFactory> {
    pub file_name: BehaviorFileName,
    pub entity: EntityTracker,
    pub asset: Option<Handle<BehaviorAsset<T>>>,
}

#[derive(Default, Resource, Deref, DerefMut)]
pub struct BehaviorTrackers<T: BehaviorFactory>(HashMap<BehaviorFileId, BehaviorTracker<T>>);

fn track_loaded_behaviors<T: BehaviorFactory>(
    mut commands: Commands,
    mut behavior_trees: Query<(Entity, &Name, &Handle<BehaviorAsset<T>>), With<BehaviorTree<T>>>,
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
                    info!("Created: {:?}", path);

                    // check if there is a tracker for this asset
                    let behavior_tracker = behavior_trackers.iter_mut().find(|(_, tracker)| {
                        if let Some(asset) = &tracker.asset {
                            asset == handle
                        } else {
                            false
                        }
                    });

                    // if there is no tracker, create one, notify file name to clients
                    if behavior_tracker.is_none() {
                        // Get asset path and build a tracker for it
                        let file_path = path.path().to_string_lossy();
                        let file_name = file_path.trim_end_matches(".bht.ron").to_owned();

                        let behavior_file_id = BehaviorFileId::new();
                        let behavior_file_name = BehaviorFileName(file_name.into());

                        behavior_trackers.insert(
                            behavior_file_id.clone(),
                            BehaviorTracker {
                                file_name: behavior_file_name.clone(),
                                entity: EntityTracker::None,
                                asset: Some(handle.clone()),
                            },
                        );

                        // server send file name to clients
                        behavior_server
                            .sender
                            .send(BehaviorProtocolServer::FileName(
                                behavior_file_id,
                                behavior_file_name,
                            ))
                            .unwrap();
                    }
                } else {
                    // Asset has no soource path, maybe was created manually
                }
            }
            AssetEvent::Modified { handle } => {
                if let Some(path) = asset_server.get_handle_path(handle) {
                    info!("Modified: {:?}", path);
                }
                for (entity, name, behavior_asset) in &mut behavior_trees {
                    if behavior_asset == handle {
                        if let Some(_asset) = behavior_assets.get(&handle) {
                            info!("Rebuilding behavior for: [{}] {}", entity.index(), name);
                            commands
                                .entity(entity)
                                .insert(BehaviorTreeReset::<T>::default());
                        }
                    }
                }
            }
            _ => {
                error!("{:?}", event);
                // Other events are not handled
            }
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
                            entity: EntityTracker::None,
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
            let entity = match behavior_tracker.entity {
                EntityTracker::Spawned(entity) => Some(entity),
                EntityTracker::Attached(entity) => Some(entity),
                EntityTracker::Inserted(entity) => Some(entity),
                EntityTracker::None => continue,
            };
            if let Some(entity) = entity {
                if let Some(behavior_asset) = &behavior_tracker.asset {
                    if let Some(behavior_assets) = world.get_resource::<Assets<BehaviorAsset<T>>>()
                    {
                        if let Some(behavior_asset) = behavior_assets.get(&behavior_asset) {
                            tracks.push((file_id.clone(), entity, behavior_asset.behavior.clone()));
                        }
                    } else {
                        error!("Failed to get behavior assets");
                    }
                } else {
                    error!("Behavior has no asset");
                }
            } else {
                error!("Behavior has no entity: {:?}", behavior_tracker.entity);
            }
        }
    } else {
        error!("Failed to get behavior trackers");
    }

    let mut behaviors_children = world.query_filtered::<&Children, With<BehaviorTree<T>>>();

    for (file_id, entity, behavior) in tracks {
        let mut root = None;
        if let Ok(children) = behaviors_children.get(world, entity) {
            root = children.first();
        }
        if let Some(root) = root {
            let mut telemetry = BehaviorTelemetry::<T>::default();
            if build_telemetry(world, *root, &mut telemetry, &behavior).is_ok() {
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

#[derive(Clone, Debug)]
struct PriorityMessage<T: BehaviorFactory> {
    priority: Duration,
    count: u64,
    msg: BehaviorProtocolClient<T>,
}

impl<T: BehaviorFactory> PartialEq for PriorityMessage<T> {
    fn eq(&self, other: &Self) -> bool {
        self.priority == other.priority
    }
}

impl<T: BehaviorFactory> Eq for PriorityMessage<T> {}

impl<T: BehaviorFactory> PartialOrd for PriorityMessage<T> {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

impl<T: BehaviorFactory> Ord for PriorityMessage<T> {
    fn cmp(&self, other: &Self) -> Ordering {
        let ordering = self.priority.cmp(&other.priority);
        match ordering {
            Ordering::Equal => Ordering::Equal,
            Ordering::Greater => Ordering::Less,
            Ordering::Less => Ordering::Greater,
        }
    }
}

#[derive(Default, Deref, DerefMut)]
struct PriorityMessageQueue<T: BehaviorFactory>(BinaryHeap<PriorityMessage<T>>);

fn update<T>(
    time: Res<Time>,
    mut commands: Commands,
    behavior_trees: Query<
        (Entity, &Name, Option<&Handle<BehaviorAsset<T>>>),
        With<BehaviorTree<T>>,
    >,
    mut behavior_assets: ResMut<Assets<BehaviorAsset<T>>>,
    mut behavior_trackers: ResMut<BehaviorTrackers<T>>,
    behavior_server: Res<BehaviorServer<T>>,
    asset_server: Res<AssetServer>,
    mut queued_msgs: Local<PriorityMessageQueue<T>>,
) where
    T: BehaviorFactory + Serialize,
{
    let priority = time.elapsed();

    // Get all messages and prepare queue
    while let Ok(msg) = behavior_server.receiver.try_recv() {
        let count = 0;
        queued_msgs.push(PriorityMessage {
            priority,
            count,
            msg,
        });
    }

    // If no messages, return
    if let Some(msg) = queued_msgs.peek() {
        if msg.priority > priority {
            return;
        }
    } else {
        return;
    }

    // Process all server messages
    while let Some(msg) = &queued_msgs.pop() {
        match &msg.msg {
            BehaviorProtocolClient::Instances(file_id) => {
                info!("Received Instances: {:?}", file_id);
                if let Some(behavior_tracker) = behavior_trackers.get_mut(&file_id) {
                    if let Some(behavior_asset) = &behavior_tracker.asset {
                        let remote_entities: Vec<protocol::RemoteEntity> = behavior_trees
                            .iter()
                            .filter_map(|(entity, name, other_behavior_asset)| {
                                if other_behavior_asset == Some(behavior_asset) {
                                    Some(protocol::RemoteEntity::new(
                                        entity,
                                        name.as_str().to_owned(),
                                    ))
                                } else {
                                    None
                                }
                            })
                            .collect();
                        behavior_server
                            .sender
                            .send(BehaviorProtocolServer::Instances(
                                file_id.clone(),
                                remote_entities,
                            ))
                            .unwrap();
                    }
                } else {
                    error!("Invalid file_id: {:?}", file_id);
                }
            }
            BehaviorProtocolClient::Orphans(file_id) => {
                info!("Received Instances: {:?}", file_id);
                if let Some(_behavior_tracker) = behavior_trackers.get_mut(&file_id) {
                    let remote_entities: Vec<protocol::RemoteEntity> = behavior_trees
                        .iter()
                        .filter_map(|(entity, name, behavior_asset)| {
                            if behavior_asset.is_none() {
                                Some(protocol::RemoteEntity::new(
                                    entity,
                                    name.as_str().to_owned(),
                                ))
                            } else {
                                None
                            }
                        })
                        .collect();
                    behavior_server
                        .sender
                        .send(BehaviorProtocolServer::Orphans(
                            file_id.clone(),
                            remote_entities,
                        ))
                        .unwrap();
                } else {
                    error!("Invalid file_id: {:?}", file_id);
                }
            }
            BehaviorProtocolClient::LoadFile(file_id) => {
                info!("Received LoadFile: {:?}", file_id);
                if let Some(behavior_tracker) = behavior_trackers.get_mut(&file_id) {
                    // check if behavior is already loaded
                    if let Some(behavior_asset) = &behavior_tracker.asset {
                        if let Some(behavior_asset) = behavior_assets.get(&behavior_asset) {
                            behavior_server
                                .sender
                                .send(BehaviorProtocolServer::FileLoaded(
                                    file_id.clone(),
                                    behavior_asset.behavior.clone(),
                                ))
                                .unwrap();
                        } else {
                            // asset not ready, check again later
                            queued_msgs.push(PriorityMessage {
                                priority: msg.priority + Duration::from_secs(1),
                                count: msg.count + 1,
                                msg: msg.msg.clone(),
                            });
                        }
                    }
                    // if not loaded, load and get a handle to asset
                    else {
                        let behavior_handle: Handle<BehaviorAsset<T>> = asset_server
                            .load(format!("{}.bht.ron", *behavior_tracker.file_name).as_str());
                        behavior_tracker.asset = Some(behavior_handle);
                        // check again later
                        queued_msgs.push(PriorityMessage {
                            priority: msg.priority + Duration::from_secs(1),
                            count: msg.count + 1,
                            msg: msg.msg.clone(),
                        });
                    }
                } else {
                    error!("Invalid file_id: {:?}", file_id);
                }
            }
            BehaviorProtocolClient::SaveFile(file_id, file_name, file_data) => {
                info!("Received SaveFile: {:?} {}", file_id, **file_name);
                let file_data =
                    ron::ser::to_string_pretty(&file_data, ron::ser::PrettyConfig::default());
                match file_data {
                    Ok(file_data) => {
                        // if we have a tracker, update the file_name
                        if let Some(behavior_tracker) = behavior_trackers.get_mut(&file_id) {
                            behavior_tracker.file_name = file_name.clone();
                        }
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
            }
            BehaviorProtocolClient::Start(file_id, file_name, start_option, behavior) => {
                info!(
                    "Received Start: {:?} {} {:?} behavior: {}",
                    file_id,
                    **file_name,
                    start_option,
                    behavior.is_some()
                );
                let mut behavior_tracker = None;
                let mut behavior_asset = None;

                // use the behavior passed in
                if let Some(behavior) = behavior {
                    // create a new asset to hold the behavior
                    let a_behavior_asset = BehaviorAsset::<T> {
                        behavior: behavior.clone(),
                    };
                    let handle = behavior_assets.add(a_behavior_asset);
                    behavior_asset = Some(handle.clone());
                }

                // if we have a tracker for this behavior, use it
                if let Some(a_behavior_tracker) = behavior_trackers.get_mut(&file_id) {
                    // if no behavior was passed in, use the one from the tracker
                    if behavior.is_none() {
                        behavior_asset = a_behavior_tracker.asset.clone();
                    }
                    behavior_tracker = Some(a_behavior_tracker);
                }
                // we dont have a tracker for this behavior, lets create one
                else {
                    if let Some(behavior) = behavior {
                        let a_behavior_asset = BehaviorAsset::<T> {
                            behavior: behavior.clone(),
                        };
                        let handle = behavior_assets.add(a_behavior_asset);
                        behavior_asset = Some(handle.clone());
                        let a_behavior_tracker = BehaviorTracker::<T> {
                            file_name: file_name.clone(),
                            asset: Some(handle.clone()),
                            entity: EntityTracker::None,
                        };
                        behavior_trackers.insert(file_id.clone(), a_behavior_tracker);
                        behavior_tracker = behavior_trackers.get_mut(&file_id);
                    }
                }

                if let (Some(behavior_tracker), Some(behavior_asset)) =
                    (behavior_tracker, behavior_asset)
                {
                    behavior_tracker.entity = EntityTracker::None;

                    match start_option {
                        // spawn behavior tree
                        StartOption::Spawn => {
                            let entity = commands
                                .spawn((
                                    Name::new(format!("BHT: {}", **file_name)),
                                    behavior_asset.clone(),
                                    BehaviorTree::<T>::default(),
                                    BehaviorTreeReset::<T>::default(),
                                ))
                                .id();
                            println!("Spawned entity: {:?}", entity);
                            behavior_tracker.entity = EntityTracker::Spawned(entity);
                        }
                        // attach to behavior tree
                        StartOption::Attach(remote_entity) => {
                            let entity = remote_entity.to_entity();
                            if behavior.is_some() {
                                commands
                                    .entity(entity)
                                    .insert(behavior_asset.clone())
                                    .insert(BehaviorTreeReset::<T>::default());
                            }
                            behavior_tracker.entity = EntityTracker::Attached(entity);
                        }
                        // insert behavior asset
                        StartOption::Insert(remote_entity) => {
                            let entity = remote_entity.to_entity();
                            commands
                                .entity(entity)
                                .insert(behavior_asset.clone())
                                .insert(BehaviorTreeReset::<T>::default());
                            behavior_tracker.entity = EntityTracker::Inserted(entity);
                        }
                    };

                    behavior_server
                        .sender
                        .send(BehaviorProtocolServer::Started(file_id.clone()))
                        .unwrap();
                } else {
                    error!("Failed to build behavior tree for file_id: {:?}", file_id);
                }
            }
            BehaviorProtocolClient::Stop(file_id, stop_option) => {
                info!("Received Stop: {:?}", file_id);
                if let Some(behavior_tracker) = behavior_trackers.get_mut(&file_id) {
                    let entity = match behavior_tracker.entity {
                        EntityTracker::Spawned(entity) => Some(entity),
                        EntityTracker::Attached(entity) => Some(entity),
                        EntityTracker::Inserted(entity) => Some(entity),
                        EntityTracker::None => None,
                    };

                    if let Some(entity) = entity {
                        match stop_option {
                            StopOption::Despawn => {
                                commands.entity(entity).despawn_recursive();
                            }
                            StopOption::Detach => {}
                            StopOption::Remove => {
                                commands.entity(entity).clear_children();
                                commands.entity(entity).remove::<Handle<BehaviorAsset<T>>>();
                            }
                        }
                    }

                    behavior_tracker.entity = EntityTracker::None;
                    behavior_server
                        .sender
                        .send(BehaviorProtocolServer::Stopped(file_id.clone()))
                        .unwrap();
                } else {
                    error!("Invalid file_id: {:?}", file_id);
                }
            }
        }

        if let Some(msg) = queued_msgs.peek() {
            if msg.priority > priority {
                break;
            }
        }
    }
}
