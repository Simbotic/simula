use bevy::{prelude::*, tasks::AsyncComputeTaskPool};
use crossbeam_channel::{bounded, Receiver};
use serde::{Deserialize, Serialize};
use surrealdb::{
    engine::remote::ws::{Client, Ws},
    opt::auth::Root,
    sql::Thing,
    Surreal,
};

pub struct SurrealPlugin;

impl Plugin for SurrealPlugin {
    fn build(&self, app: &mut App) {
        app.register_type::<SurrealClient>()
            .add_startup_system(client_connector)
            .add_system(client_receiver);
    }
}

#[derive(Reflect, Resource, Debug, Default, Clone)]
#[reflect(Resource, Debug)]
pub struct SurrealClient {
    #[reflect(ignore)]
    client: Option<Surreal<Client>>,
    pub endpoint: String,
    pub version: String,
    pub namespace: String,
    pub database: String,
    pub username: String,
    pub password: String,
}

#[derive(Resource, Deref)]
struct SurrealClientReceiver(Receiver<SurrealClient>);

fn client_connector(mut commands: Commands) {
    let (sender, receiver) = bounded(1);
    commands.insert_resource(SurrealClientReceiver(receiver));

    let thread_pool = AsyncComputeTaskPool::get();
    let _ = thread_pool.spawn(async move {
        let address = "127.0.0.1:8000";

        info!("Connecting to SurrealDB: {}", address);
        let connect = Surreal::new::<Ws>(address).await;
        if let Ok(client) = connect {
            info!("Connected to SurrealDB");

            let version = if let Ok(version) = client.version().await {
                version.to_string()
            } else {
                "unknown".into()
            };

            let client_resource = SurrealClient {
                client: Some(client.clone()),
                endpoint: address.to_string(),
                version,
                namespace: "default".into(),
                database: "default".into(),
                username: "root".into(),
                password: "root".into(),
                ..default()
            };

            let _ = client
                .signin(Root {
                    username: &client_resource.username,
                    password: &client_resource.password,
                })
                .await;
            let _ = client
                .use_ns(&client_resource.namespace)
                .use_db(&client_resource.database)
                .await;

            sender.send(client_resource).unwrap();
        } else if let Err(e) = connect {
            error!("Failed to connect to SurrealDB: {:?}", e);
        }
    });
}

fn client_receiver(mut commands: Commands, surreal_client_receiver: Res<SurrealClientReceiver>) {
    if let Some(client) = surreal_client_receiver.try_recv().ok() {
        info!("Surreal client received");

        commands.insert_resource(client.clone());

        if let Some(client) = client.client {
            let thread_pool = AsyncComputeTaskPool::get();
            let _ = thread_pool.spawn(async move {
                let created: Result<Vec<Record>, surrealdb::Error> = client
                    .create("person")
                    .content(Person {
                        title: "Researcher",
                        name: Name {
                            first: "Alex",
                            last: "Rozgo",
                        },
                        simulation: true,
                    })
                    .await;
                if let Ok(created) = created {
                    info!("Created: {:?}", created);
                } else if let Err(e) = created {
                    error!("{:?}", e);
                }
            });
        }
    }
}

#[derive(Debug, Serialize)]
struct Name<'a> {
    first: &'a str,
    last: &'a str,
}

#[derive(Debug, Serialize)]
struct Person<'a> {
    title: &'a str,
    name: Name<'a>,
    simulation: bool,
}

#[derive(Debug, Deserialize)]
struct Record {
    #[allow(dead_code)]
    id: Thing,
}
