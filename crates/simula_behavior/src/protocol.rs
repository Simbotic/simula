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

#[derive(Debug, Clone, Serialize, Deserialize, Reflect, FromReflect, PartialEq, Hash, Eq)]
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
}

#[derive(
    Debug, Clone, Serialize, Deserialize, Deref, DerefMut, Reflect, FromReflect, PartialEq, Hash, Eq,
)]
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

#[derive(
    Debug, Clone, Serialize, Deserialize, Deref, DerefMut, Reflect, FromReflect, PartialEq, Hash, Eq,
)]
pub struct BehaviorFileName(pub Cow<'static, str>);

pub enum BehaviorProtocolClient<T: BehaviorFactory> {
    Instances(BehaviorFileId),
    LoadFile(BehaviorFileId),
    SaveFile(BehaviorFileId, BehaviorFileName, Behavior<T>),
    Start(BehaviorFileId, BehaviorFileName, StartOption, Option<Behavior<T>>),
    Stop(BehaviorFileId),
}

pub enum BehaviorProtocolServer<T: BehaviorFactory> {
    FileName(BehaviorFileId, BehaviorFileName),
    Instances(BehaviorFileId, Vec<RemoteEntity>),
    FileLoaded(BehaviorFileId, Behavior<T>),
    FileSaved(BehaviorFileId),
    Started(BehaviorFileId),
    Stopped(BehaviorFileId),
    Telemetry(BehaviorFileId, BehaviorTelemetry<T>),
}

#[derive(Debug, Default)]
pub struct BehaviorTelemetry<T: BehaviorFactory>(
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
