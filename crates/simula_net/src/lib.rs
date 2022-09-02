use bevy::{prelude::*, tasks::IoTaskPool};
use simula_socket::WebRtcSocket;

pub struct NetPlugin;

impl Plugin for NetPlugin {
    fn build(&self, app: &mut App) {
        app.add_startup_system(setup);
    }
}

fn setup(mut commands: Commands) {
    let room_id: String = "Yeehah!".into();
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
