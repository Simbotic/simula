use crate::{Behavior, BehaviorFactory};
use bevy::{prelude::*, utils::Uuid};
use crossbeam_channel::{Receiver, Sender};
use serde::{Deserialize, Serialize};
use std::borrow::Cow;

#[derive(Resource)]
pub struct BehaviorClient<T: BehaviorFactory> {
    pub sender: Sender<BehaviorProtocolClient<T>>,
    pub receiver: Receiver<BehaviorProtocolServer<T>>,
}

#[derive(Resource)]
pub struct BehaviorServer<T: BehaviorFactory> {
    pub sender: Sender<BehaviorProtocolServer<T>>,
    pub receiver: Receiver<BehaviorProtocolClient<T>>,
}

#[derive(Debug, Clone, Serialize, Deserialize, Reflect, PartialEq, Hash, Eq)]
pub struct RemoteEntity {
    pub bits: u64,
    pub name: Cow<'static, str>,
}

impl RemoteEntity {
    pub fn new(entity: Entity, name: impl Into<Cow<'static, str>>) -> Self {
        Self {
            bits: entity.to_bits(),
            name: name.into(),
        }
    }

    pub fn to_entity(&self) -> Entity {
        Entity::from_bits(self.bits)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub enum StartOption {
    Spawn,
    Attach(RemoteEntity),
    Insert(RemoteEntity),
}

#[derive(Debug, Clone, PartialEq)]
pub enum StopOption {
    Despawn,
    Detach,
    Remove,
}

#[derive(Debug, Clone, Serialize, Deserialize, Deref, DerefMut, Reflect, PartialEq, Hash, Eq)]
pub struct BehaviorFileId(Cow<'static, str>);

impl BehaviorFileId {
    pub fn new() -> Self {
        Self(
            Uuid::new_v4()
                .simple()
                .to_string()
                .chars()
                .take(5)
                .collect::<String>()
                .into(),
        )
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, Deref, DerefMut, Reflect, PartialEq, Hash, Eq)]
pub struct BehaviorFileName(pub Cow<'static, str>);

#[derive(Debug, Clone)]
pub enum BehaviorProtocolClient<T: BehaviorFactory> {
    /// Request instances running behavior
    Instances(BehaviorFileId),
    /// Request orphans, instances that have behavior trees but no behavior
    Orphans(BehaviorFileId),
    /// Request file to be loaded
    LoadFile(BehaviorFileId),
    /// Request file to be saved
    SaveFile(BehaviorFileId, BehaviorFileName, Behavior<T>),
    /// Request behavior to be started
    Start(
        BehaviorFileId,
        BehaviorFileName,
        StartOption,
        Option<Behavior<T>>,
    ),
    /// Request behavior to be stopped
    Stop(BehaviorFileId, StopOption),
}

pub enum BehaviorProtocolServer<T: BehaviorFactory> {
    /// Behavior file listed
    FileName(BehaviorFileId, BehaviorFileName),
    /// Instances running behavior
    Instances(BehaviorFileId, Vec<RemoteEntity>),
    /// Instances that have behavior trees but no behavior
    Orphans(BehaviorFileId, Vec<RemoteEntity>),
    /// Behavior file loaded
    FileLoaded(BehaviorFileId, Behavior<T>),
    /// Behavior file saved
    FileSaved(BehaviorFileId),
    /// Behavior started
    Started(BehaviorFileId),
    /// Behavior stopped
    Stopped(BehaviorFileId),
    /// Behavior telemetry
    Telemetry(BehaviorFileId, BehaviorTelemetry<T>),
}

#[derive(Debug, Default)]
pub struct BehaviorTelemetry<T: BehaviorFactory>(
    pub Option<RemoteEntity>,
    pub BehaviorState,
    pub Option<T>,
    pub Vec<BehaviorTelemetry<T>>,
);

#[derive(Debug, Default, Clone, Copy)]
pub enum BehaviorState {
    #[default]
    None,
    Running,
    Cursor,
    Success,
    Failure,
}
