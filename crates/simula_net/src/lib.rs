use bevy::{prelude::*, tasks::IoTaskPool};
use simula_socket::WebRtcSocket;

#[derive(Default, Reflect, Component)]
#[reflect(Component)]
pub struct NetPeer {
    pub id: String,
}

pub struct NetPlugin;

impl Plugin for NetPlugin {
    fn build(&self, app: &mut App) {
        app.register_type::<NetPeer>()
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

    commands.insert_resource(Some(socket));
}

fn run(
    mut commands: Commands,
    mut socket: ResMut<Option<WebRtcSocket>>,
    peers: Query<(Entity, &NetPeer)>,
) {
    let socket = socket.as_mut();
    if let Some(socket) = socket {
        let new_peers = socket.accept_new_connections();
        for peer in new_peers {
            info!("New peer: {}", peer);
        }

        let net_peers = socket.connected_peers();

        // Remove old peers from world
        // TODO: in a quick test it didnt work
        for (entity, peer) in &peers {
            if !net_peers.contains(&peer.id) {
                warn!("SHOULD REMOVE peer");
                commands.entity(entity).despawn_recursive();
            }
        }

        // Add new peers to world
        for net_peer in net_peers {
            if peers
                .iter()
                .find(|(_, peer)| peer.id == *net_peer)
                .is_none()
            {
                commands
                    .spawn()
                    .insert(NetPeer {
                        id: net_peer.clone(),
                    })
                    .insert(Name::new(format!(
                        "Peer: [{}]",
                        net_peer.get(0..4).unwrap_or_default()
                    )));
            }
        }
    }
}
