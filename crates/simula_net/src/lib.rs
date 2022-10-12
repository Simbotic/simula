use bevy::{prelude::*, tasks::IoTaskPool, utils::Uuid};
use serde::{Deserialize, Serialize};
use simula_socket::WebRtcSocket;
use std::collections::hash_map::DefaultHasher;
use std::fmt::Debug;
use std::hash::{Hash, Hasher};

#[derive(Default, Reflect, Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct PeerId {
    #[reflect(ignore)]
    uuid: Uuid,
    id: String,
}

impl PeerId {
    pub fn new(id: &Uuid) -> Self {
        Self {
            uuid: id.clone(),
            id: id.to_string().get(0..4).unwrap_or_default().to_string(),
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

impl Plugin for NetPlugin {
    fn build(&self, app: &mut App) {
        app.insert_resource(Messages::default())
            .register_type::<RemotePeer>()
            .register_type::<LocalPeer>()
            .register_type::<Replicate>()
            .register_type::<Proxy>()
            .register_type::<PeerId>()
            .add_system_to_stage(CoreStage::PreUpdate, extract_messages)
            .add_system_to_stage(CoreStage::PostUpdate, cleanup_proxies)
            .add_startup_system(setup)
            .add_system(run);
    }
}

fn setup(mut commands: Commands) {
    let room_id: String = "simula".into();
    let room_host: String = "ws://127.0.0.1:3536".into();

    let room_url = format!("{}/{}", room_host, room_id);
    info!("Connecting to Simula server: {:?}", room_url);
    let (socket, message_loop) = WebRtcSocket::new(room_url);

    // The message loop needs to be awaited, or nothing will happen.
    // We do this here using bevy's task system.
    let task_pool = IoTaskPool::get();
    task_pool.spawn(message_loop).detach();

    let peer_id = PeerId::new(socket.id());

    commands.insert_resource(Some(socket));

    commands
        .spawn()
        .insert(LocalPeer {
            id: peer_id.clone(),
        })
        .insert(Name::new(format!("Peer: Local ({})", peer_id.id)));
}

fn run(
    mut commands: Commands,
    mut socket: ResMut<Option<WebRtcSocket>>,
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
            if peers
                .iter()
                .find(|(_, peer)| peer.id.uuid == *net_peer)
                .is_none()
            {
                commands
                    .spawn()
                    .insert(RemotePeer {
                        id: PeerId::new(net_peer),
                    })
                    .insert(Name::new(format!(
                        "Peer: Remote ({})",
                        net_peer.to_string().get(0..4).unwrap_or_default()
                    )));
            }
        }
    }
}

#[derive(Default)]
pub struct Messages {
    messages: Vec<(Uuid, Box<[u8]>)>,
}

pub fn extract_messages(mut socket: ResMut<Option<WebRtcSocket>>, mut messages: ResMut<Messages>) {
    if let Some(socket) = socket.as_mut() {
        messages.messages = socket.receive();
    }
}

#[derive(Debug, Clone, Component, Reflect)]
#[reflect(Component)]
pub struct Replicate {
    #[reflect(ignore)]
    pub uuid: Uuid,
    pub id: String,
    /// rate in hertz
    pub rate: f64,
    pub last_sync: f64,
    pub target: Option<PeerId>,
}

impl Default for Replicate {
    fn default() -> Self {
        let id = Uuid::new_v4();
        Self {
            uuid: id.clone(),
            id: id.to_string().get(0..4).unwrap_or_default().to_string(),
            rate: 1.0,
            last_sync: 0.0,
            target: None,
        }
    }
}

#[derive(Debug, Component, Reflect, Deserialize)]
#[reflect(Component)]
pub struct Proxy {
    #[reflect(ignore)]
    pub uuid: Uuid,
    pub id: String,
    pub sender: PeerId,
}

impl Default for Proxy {
    fn default() -> Self {
        let id = Uuid::new_v4();
        Self {
            uuid: id.clone(),
            id: id.to_string().get(0..4).unwrap_or_default().to_string(),
            sender: PeerId::new(&Uuid::new_v4()),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
enum Payload<T> {
    Replicate {
        type_id: u32,
        uuid: Uuid,
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
    time: Res<Time>,
    mut commands: Commands,
    mut socket: ResMut<Option<WebRtcSocket>>,
    messages: Res<Messages>,
    peers: Query<&RemotePeer>,
    mut syncs: Query<(&mut Replicate, &T, Option<&Name>)>,
    mut proxies: Query<(&mut T, &Proxy), Without<Replicate>>,
) where
    T: Reflect + Debug + for<'de> Deserialize<'de> + Component + Serialize + Send + Sync + 'static,
{
    let replicate_type_id = get_hash::<T>();

    if let Some(socket) = socket.as_mut() {
        // Send data to peers
        for (mut sync, data, name) in syncs.iter_mut() {
            let name = if let Some(name) = name {
                name.into()
            } else {
                "Entity".to_string()
            };
            if sync.last_sync + 1.0 / sync.rate < time.seconds_since_startup() {
                sync.last_sync = time.seconds_since_startup();
                let payload = Payload::Replicate {
                    type_id: replicate_type_id,
                    uuid: sync.uuid,
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
                            println!(
                                "Request replicate for: {:?} type: {} type_id: {}",
                                peer.id,
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
            if let Ok(message) = bincode::deserialize::<Payload<T>>(&message) {
                trace!(
                    "Received from: {} net message: {}",
                    peer_id,
                    std::any::type_name::<T>().to_string(),
                );
                match message {
                    Payload::Replicate {
                        type_id,
                        uuid,
                        data,
                        name,
                    } if type_id == replicate_type_id => {
                        let proxy = proxies.iter_mut().find(|(_, proxy)| proxy.uuid == uuid);
                        if let Some((mut proxy_data, _)) = proxy {
                            *proxy_data = data;
                        } else {
                            println!(
                                "Instantiate Proxy for: {} type: {}",
                                peer_id,
                                std::any::type_name::<T>().to_string(),
                            );
                            let id = uuid.to_string().get(0..4).unwrap_or_default().to_string();
                            commands
                                .spawn()
                                .insert(data)
                                .insert(Proxy {
                                    uuid,
                                    id: id.clone(),
                                    sender: PeerId::new(peer_id),
                                })
                                .insert(Name::new(format!("{}: Proxy ({})", name, id)));
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
    socket: Res<Option<WebRtcSocket>>,
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
