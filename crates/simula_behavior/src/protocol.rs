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

#[derive(Debug, Clone, Serialize, Deserialize, Deref, DerefMut, Reflect, FromReflect)]
pub struct BehaviorFileData(pub Cow<'static, str>);

pub enum BehaviorProtocolClient<T: BehaviorFactory> {
    Ping,
    LoadFile(BehaviorFileId),
    SaveFile(BehaviorFileId, BehaviorFileName, BehaviorFileData),
    Run(BehaviorFileId, Behavior<T>),
    Stop(BehaviorFileId),
}

pub enum BehaviorProtocolServer<T: BehaviorFactory> {
    Pong,
    FileNames(Vec<(BehaviorFileId, BehaviorFileName)>),
    File(BehaviorFileId, BehaviorFileData),
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
