use bevy::{
    prelude::*,
    tasks::IoTaskPool,
    utils::{HashMap, Uuid},
};
use serde::{Deserialize, Serialize};
use simula_socket::{WebRtcSocket, WebRtcSocketConfig};
use std::collections::hash_map::DefaultHasher;
use std::fmt::Debug;
use std::hash::{Hash, Hasher};

#[derive(
    Default, Reflect, Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize, FromReflect,
)]
pub struct PeerId {
    #[reflect(ignore)]
    uuid: Uuid,
    id: String,
}

impl PeerId {
    pub fn new(id: &Uuid) -> Self {
        Self {
            uuid: *id,
            id: id.to_string(),
        }
    }
}

#[derive(Default, Reflect, Debug, Clone, PartialEq, Eq, Hash)]
pub struct RoomId {
    #[reflect(ignore)]
    uuid: Uuid,
    id: String,
}

#[derive(Default, Reflect, Component)]
#[reflect(Component)]
pub struct RemotePeer {
    id: PeerId,
}

#[derive(Default, Reflect, Component)]
#[reflect(Component)]
pub struct LocalPeer {
    id: PeerId,
}

pub struct NetPlugin;

#[derive(Resource)]
pub struct NetPluginSettings {
    pub room_id: String,
    pub room_host: String,
    pub socket_config: Option<WebRtcSocketConfig>,
}

impl Default for NetPluginSettings {
    fn default() -> Self {
        Self {
            room_id: "simula".into(),
            room_host: "ws://127.0.0.1:3536".into(),
            socket_config: None,
        }
    }
}

impl Plugin for NetPlugin {
    fn build(&self, app: &mut App) {
        app.world
            .get_resource_or_insert_with(NetPluginSettings::default);

        app.insert_resource(ProxyCache::default())
            .insert_resource(Messages::default())
            .register_type::<RemotePeer>()
            .register_type::<LocalPeer>()
            .register_type::<Proxy>()
            .register_type::<PeerId>()
            .register_type::<NetId>()
            .add_system_to_stage(CoreStage::PreUpdate, extract_messages)
            .add_system_to_stage(CoreStage::PostUpdate, cleanup_proxies)
            .add_startup_system(setup)
            .add_system(run);
    }
}

fn setup(mut commands: Commands, settings: Res<NetPluginSettings>) {
    let room_url = format!("{}/{}", settings.room_host, settings.room_id);
    info!("Connecting to Simula server: {:?}", room_url);
    let socket;
    let message_loop;

    if let Some(socket_config) = settings.socket_config.clone() {
        (socket, message_loop) = WebRtcSocket::new_with_config(socket_config);
    } else {
        (socket, message_loop) = WebRtcSocket::new(room_url);
    }

    // The message loop needs to be awaited, or nothing will happen.
    // We do this here using bevy's task system.
    let task_pool = IoTaskPool::get();
    task_pool.spawn(message_loop).detach();

    let peer_id = PeerId::new(socket.id());

    commands.insert_resource(socket);

    commands
        .spawn_empty()
        .insert(LocalPeer {
            id: peer_id.clone(),
        })
        .insert(Name::new(format!(
            "Peer: Local ({})",
            peer_id.id.get(0..8).unwrap_or_default()
        )));
}

fn run(
    mut commands: Commands,
    mut socket: Option<ResMut<WebRtcSocket>>,
    peers: Query<(Entity, &RemotePeer)>,
) {
    let socket = socket.as_mut();
    if let Some(socket) = socket {
        let new_peers = socket.accept_new_connections();
        for peer in new_peers {
            info!("New peer: {}", peer);
        }

        let _disconnected_peers = socket.disconnected_peers();

        let connected_peers = socket.connected_peers();

        // Remove old peers from world
        for (entity, peer) in &peers {
            if !connected_peers.contains(&peer.id.uuid) {
                commands.entity(entity).despawn_recursive();
            }
        }

        // Add new peers to world
        for net_peer in connected_peers {
            if !peers.iter().any(|(_, peer)| peer.id.uuid == *net_peer) {
                commands
                    .spawn_empty()
                    .insert(RemotePeer {
                        id: PeerId::new(net_peer),
                    })
                    .insert(Name::new(format!(
                        "Peer: Remote ({})",
                        net_peer.to_string().get(0..8).unwrap_or_default()
                    )));
            }
        }
    }
}

#[derive(Default, Deref, DerefMut, Resource)]
pub struct ProxyCache(HashMap<Uuid, Entity>);

#[derive(Default, Resource)]
pub struct Messages {
    messages: Vec<(Uuid, Box<[u8]>)>,
}

pub fn extract_messages(mut socket: Option<ResMut<WebRtcSocket>>, mut messages: ResMut<Messages>) {
    messages.messages.clear();
    if let Some(socket) = socket.as_mut() {
        let mut message = socket.receive_one();
        while let Ok(Some((peer, data))) = message {
            messages.messages.push((peer, data));
            message = socket.receive_one();
            debug!("Received message from peer: {}", peer);
        }
    }
}

#[derive(Debug, Clone, Component, Reflect)]
#[reflect(Component)]
pub struct NetId {
    #[reflect(ignore)]
    pub uuid: Uuid,
    pub id: String,
}

impl Default for NetId {
    fn default() -> Self {
        let id = Uuid::new_v4();
        Self {
            uuid: id,
            id: id.to_string(),
        }
    }
}

#[derive(Debug, Clone, Component, Reflect)]
#[reflect(Component)]
pub struct Replicate<T>
where
    T: 'static + Reflect,
{
    /// rate in hertz
    pub rate: f64,
    pub last_sync: f64,
    pub target: Option<PeerId>,
    #[reflect(ignore)]
    pub phantom: std::marker::PhantomData<T>,
}

impl<T> Default for Replicate<T>
where
    T: 'static + Reflect,
{
    fn default() -> Self {
        Self {
            rate: 1.0,
            last_sync: 0.0,
            target: None,
            phantom: std::marker::PhantomData,
        }
    }
}

#[derive(Debug, Component, Reflect, Deserialize)]
#[reflect(Component)]
pub struct Proxy {
    pub sender: PeerId,
}

impl Default for Proxy {
    fn default() -> Self {
        Self {
            sender: PeerId::new(&Uuid::default()),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
enum Payload<T> {
    Replicate {
        type_id: u32,
        net_id: Uuid,
        data: T,
        name: String,
    },
}

fn get_hash<T: Reflect>() -> u32 {
    let mut hasher = DefaultHasher::new();
    std::any::type_name::<T>().hash(&mut hasher);
    hasher.finish() as u32
}

pub fn replicate<T>(
    mut cache: ResMut<ProxyCache>,
    time: Res<Time>,
    mut commands: Commands,
    mut socket: Option<ResMut<WebRtcSocket>>,
    messages: Res<Messages>,
    peers: Query<&RemotePeer>,
    mut syncs: Query<(&NetId, &mut Replicate<T>, &T, Option<&Name>)>,
    mut proxies: Query<(&mut T, &NetId, &Proxy), Without<Replicate<T>>>,
) where
    T: Reflect + Debug + for<'de> Deserialize<'de> + Component + Serialize + Send + Sync + 'static,
{
    let replicate_type_id = get_hash::<T>();

    if let Some(socket) = socket.as_mut() {
        // Send data to peers
        for (net_id, mut sync, data, name) in syncs.iter_mut() {
            let name = if let Some(name) = name {
                name.into()
            } else {
                "Entity".to_string()
            };
            if sync.last_sync + 1.0 / sync.rate < time.elapsed_seconds_f64() {
                sync.last_sync = time.elapsed_seconds_f64();
                let payload = Payload::Replicate {
                    type_id: replicate_type_id,
                    net_id: net_id.uuid,
                    data: data.clone(),
                    name,
                };
                if let Ok(packet) = bincode::serialize(&payload) {
                    for peer in peers.iter() {
                        let mut should_send = false;
                        if let Some(target) = &sync.target {
                            if target == &peer.id {
                                should_send = true;
                            }
                        } else {
                            should_send = true;
                        }
                        if should_send {
                            trace!(
                                "Request replicate peer_id: {} net_id: {} component: {} type_id: {}",
                                peer.id.id,
                                net_id.id,
                                std::any::type_name::<T>().to_string(),
                                replicate_type_id
                            );
                            socket.send(packet.clone().into(), peer.id.uuid);
                        }
                    }
                } else {
                    error!(
                        "Failed to serialize net message: {}",
                        std::any::type_name::<T>().to_string()
                    );
                }
            }
        }

        // Receive data from peers
        for (peer_id, message) in messages.messages.iter() {
            if let Ok(message) = bincode::deserialize::<Payload<T>>(message) {
                trace!(
                    "Received from: {} net message: {}",
                    peer_id,
                    std::any::type_name::<T>().to_string(),
                );
                match message {
                    Payload::Replicate {
                        type_id,
                        net_id,
                        data,
                        name,
                    } if type_id == replicate_type_id => {
                        let proxy = proxies
                            .iter_mut()
                            .find(|(_, proxy_net_id, _proxy)| proxy_net_id.uuid == net_id);
                        if let Some((mut proxy_data, _net_id, _proxy)) = proxy {
                            *proxy_data = data;
                        } else {
                            let net_id_label =
                                net_id.to_string().get(0..8).unwrap_or_default().to_string();
                            let cached;
                            let entity = if let Some(entity) = cache.get(&net_id) {
                                cached = true;
                                *entity
                            } else {
                                cached = false;
                                let entity = commands
                                    .spawn_empty()
                                    .insert(Proxy {
                                        sender: PeerId::new(peer_id),
                                    })
                                    .insert(NetId {
                                        uuid: net_id,
                                        id: net_id.to_string(),
                                    })
                                    .insert(Name::new(format!(
                                        "{}: Proxy ({})",
                                        name, net_id_label
                                    )))
                                    .id();
                                // TODO: Clear cache at some point
                                cache.insert(net_id, entity);
                                entity
                            };
                            commands.entity(entity).insert(data);
                            debug!(
                                "Replicated peer_id: {} net_id: {} component: {} type_id: {} cached: {}",
                                peer_id.to_string().get(0..8).unwrap_or_default(),
                                net_id_label,
                                std::any::type_name::<T>().to_string(),
                                type_id,
                                cached
                            );
                        }
                    }
                    _ => {}
                }
            } else {
                error!(
                    "Failed to deserialize net message: {}",
                    std::any::type_name::<T>().to_string()
                );
            }
        }
    }
}

fn cleanup_proxies(
    mut commands: Commands,
    proxies: Query<(Entity, &Proxy)>,
    socket: Option<ResMut<WebRtcSocket>>,
) {
    if let Some(socket) = socket.as_ref() {
        let connected_peers = socket.connected_peers();
        for (entity, proxy) in proxies.iter() {
            if !connected_peers.contains(&proxy.sender.uuid) {
                commands.entity(entity).despawn_recursive();
            }
        }
    }
}
