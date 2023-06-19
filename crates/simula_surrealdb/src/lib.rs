use bevy::prelude::*;
#[cfg(target_arch = "wasm32")]
use bevy::tasks::AsyncComputeTaskPool;
use crossbeam_channel::{bounded, Receiver};
#[cfg(not(target_arch = "wasm32"))]
use lazy_static::lazy_static;
use std::future::Future;
#[cfg(not(target_arch = "wasm32"))]
use tokio::runtime::Runtime;

pub mod client;
pub mod server;

pub struct SurrealPlugin;

#[derive(Reflect, Resource, Debug, Default)]
pub struct SurrealResource {
    #[cfg(not(target_arch = "wasm32"))]
    #[reflect(ignore)]
    pub tokio: Option<tokio::runtime::Runtime>,
}

impl Plugin for SurrealPlugin {
    fn build(&self, _app: &mut App) {}
}

#[cfg(not(target_arch = "wasm32"))]
lazy_static! {
    pub static ref TOKIO_RT: Runtime = Runtime::new().expect("Failed to create Tokio runtime");
}

#[cfg(not(target_arch = "wasm32"))]
pub fn queue_task<T>(future: impl Future<Output = T> + Send + 'static) -> Receiver<T>
where
    T: Send + 'static,
{
    let (sender, receiver) = bounded(1);
    TOKIO_RT.spawn(async move {
        let result = future.await;
        sender.send(result).unwrap();
    });
    receiver
}

#[cfg(target_arch = "wasm32")]
pub fn queue_task<T>(future: impl Future<Output = T> + Send + 'static) -> Receiver<T>
where
    T: Send + 'static,
{
    let (sender, receiver) = bounded(1);
    let thread_pool = AsyncComputeTaskPool::get();
    thread_pool.spawn(async move {
        let result = future.await;
        sender.send(result).unwrap();
    });
    receiver
}
