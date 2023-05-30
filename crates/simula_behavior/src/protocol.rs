use crate::{inspector::graph::MyEditorState, BehaviorFactory};
use bevy::{prelude::*, utils::Uuid};
use crossbeam_channel::{Receiver, Sender};
use serde::{Deserialize, Serialize};

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
pub struct BehaviorFileId(pub String);

impl BehaviorFileId {
    pub fn new() -> Self {
        Self(
            Uuid::new_v4()
                .simple()
                .to_string()
                .chars()
                .take(5)
                .collect::<String>(),
        )
    }
}

#[derive(
    Debug, Clone, Serialize, Deserialize, Deref, DerefMut, Reflect, FromReflect, PartialEq, Hash, Eq,
)]
pub struct BehaviorFileName(pub String);

#[derive(Debug, Clone, Serialize, Deserialize, Deref, DerefMut, Reflect, FromReflect)]
pub struct BehaviorFileData(pub String);

pub enum BehaviorProtocolClient<T: BehaviorFactory> {
    Ping,
    LoadFile(BehaviorFileId),
    SaveFile((BehaviorFileId, BehaviorFileData)),
    Update(MyEditorState<T>),
}

pub enum BehaviorProtocolServer<T: BehaviorFactory> {
    Pong,
    BehaviorFileNames(Vec<(BehaviorFileId, BehaviorFileName)>),
    BehaviorFile((BehaviorFileId, BehaviorFileData)),
    Update(MyEditorState<T>),
}
