use crate::{
    prelude::*,
    protocol::{
        BehaviorFileData, BehaviorFileId, BehaviorFileName, BehaviorProtocolClient,
        BehaviorProtocolServer, BehaviorServer, BehaviorState, BehaviorTelemetry,
    },
};
use bevy::{prelude::*, utils::HashMap};

#[derive(Default)]
pub struct BehaviorServerPlugin<T: BehaviorFactory>(pub std::marker::PhantomData<T>);

impl<T> Plugin for BehaviorServerPlugin<T>
where
    T: BehaviorFactory,
{
    fn build(&self, app: &mut App) {
        app.insert_resource(BehaviorTrackers::<T>::default())
            .add_startup_system(setup::<T>)
            .add_system(update::<T>)
            .add_system(update_telemetry::<T>);
    }
}

pub struct BehaviorTracker<T: BehaviorFactory> {
    file_name: BehaviorFileName,
    entity: Option<Entity>,
    telemetry: bool,
    behavior: Option<Behavior<T>>,
}

#[derive(Default, Resource, Deref, DerefMut)]
pub struct BehaviorTrackers<T: BehaviorFactory>(HashMap<BehaviorFileId, BehaviorTracker<T>>);

fn setup<T: BehaviorFactory>(
    mut behavior_trackers: ResMut<BehaviorTrackers<T>>,
    behavior_server: Res<BehaviorServer<T>>,
) {
    let dir_path = "assets/inspector";

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
                    let file_name = file_name.trim_end_matches(".bht.ron");
                    behavior_trackers.insert(
                        BehaviorFileId::new(),
                        BehaviorTracker {
                            file_name: BehaviorFileName(file_name.to_string().into()),
                            entity: None,
                            telemetry: false,
                            behavior: None,
                        },
                    );
                }
            }
        }
    }

    behavior_server
        .sender
        .send(BehaviorProtocolServer::FileNames(
            behavior_trackers
                .iter()
                .map(|(tracker_id, tracker)| (tracker_id.clone(), tracker.file_name.clone()))
                .collect(),
        ))
        .unwrap();
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
    if let Some(mut behavior_trackers) = world.get_resource_mut::<BehaviorTrackers<T>>() {
        for (file_id, behavior_tracker) in behavior_trackers.iter_mut() {
            if behavior_tracker.telemetry {
                if let Some(entity) = behavior_tracker.entity {
                    if let Some(behavior) = behavior_tracker.behavior.clone() {
                        tracks.push((file_id.clone(), entity, behavior));
                    }
                }
            }
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
    } else {
        error!("Failed to get behavior trackers");
    }
}

fn update<T: BehaviorFactory>(
    mut commands: Commands,
    behavior_trees: Query<&BehaviorTree<T>>,
    mut behavior_trackers: ResMut<BehaviorTrackers<T>>,
    behavior_server: Res<BehaviorServer<T>>,
) {
    while let Ok(client_msg) = behavior_server.receiver.try_recv() {
        match client_msg {
            BehaviorProtocolClient::LoadFile(file_id) => {
                if let Some(behavior_tracker) = behavior_trackers.get_mut(&file_id) {
                    let file_name = behavior_tracker.file_name.clone();
                    let dir_path = "assets/inspector";
                    let file_ext = "bht.ron";
                    let file_path = format!("{}/{}.{}", dir_path, *file_name, file_ext);
                    if let Ok(file_data) = std::fs::read_to_string(&file_path) {
                        behavior_server
                            .sender
                            .send(BehaviorProtocolServer::File(
                                file_id,
                                BehaviorFileData(file_data.into()),
                            ))
                            .unwrap();
                    } else {
                        error!("Failed to read file: {}", &file_path);
                    }
                } else {
                    error!("Invalid file_id: {:?}", file_id);
                }
            }
            BehaviorProtocolClient::SaveFile(file_id, file_name, file_data) => {
                behavior_trackers.insert(
                    file_id.clone(),
                    BehaviorTracker {
                        file_name: file_name.clone(),
                        entity: None,
                        telemetry: false,
                        behavior: None,
                    },
                );
                let dir_path = "assets/inspector";
                let file_ext = "bht.ron";
                let file_path = format!("{}/{}.{}", dir_path, *file_name, file_ext);
                std::fs::write(&file_path, file_data.0.as_ref()).unwrap();
                info!("Saved file: {}", &file_path);
                behavior_server
                    .sender
                    .send(BehaviorProtocolServer::FileSaved(file_id))
                    .unwrap();
            }
            BehaviorProtocolClient::Run(file_id, behavior) => {
                if !behavior_trackers.contains_key(&file_id) {
                    behavior_trackers.insert(
                        file_id.clone(),
                        BehaviorTracker {
                            file_name: BehaviorFileName((*file_id).clone().into()),
                            entity: None,
                            telemetry: false,
                            behavior: None,
                        },
                    );
                };

                if let Some(behavior_tracker) = behavior_trackers.get_mut(&file_id) {
                    behavior_tracker.behavior = Some(behavior.clone());

                    let behavior_tree_entity = {
                        if let Some(behavior_tree_entity) = behavior_tracker.entity {
                            if let Ok(behavior_tree) = behavior_trees.get(behavior_tree_entity) {
                                if let Some(root) = behavior_tree.root {
                                    commands.entity(root).despawn_recursive();
                                }
                            }
                            behavior_tree_entity
                        } else {
                            let behavior_tree_entity = commands
                                .spawn(Name::new(format!("BT: {}", *behavior_tracker.file_name)))
                                .id();
                            behavior_tracker.entity = Some(behavior_tree_entity);
                            behavior_tree_entity
                        }
                    };

                    let mut behavior_tree = BehaviorTree::<T>::default();
                    let root = commands.spawn(BehaviorCursor::Delegate).id();
                    BehaviorTree::insert_tree(
                        behavior_tree_entity,
                        root,
                        None,
                        &mut commands,
                        &behavior,
                    );
                    behavior_tree.root = Some(root);
                    commands
                        .entity(behavior_tree_entity)
                        .insert(behavior_tree)
                        .add_child(root);
                    behavior_server
                        .sender
                        .send(BehaviorProtocolServer::Started(file_id))
                        .unwrap();
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
                        .send(BehaviorProtocolServer::Stopped(file_id))
                        .unwrap();
                } else {
                    error!("Invalid file_id: {:?}", file_id);
                }
            }
            BehaviorProtocolClient::Telemetry(file_id, enable) => {
                if let Some(behavior_tracker) = behavior_trackers.get_mut(&file_id) {
                    behavior_tracker.telemetry = enable;
                } else {
                    error!("Invalid file_id: {:?}", file_id);
                }
            }
        }
    }
}
