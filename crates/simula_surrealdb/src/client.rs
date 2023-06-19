use crate::queue_task;
use bevy::prelude::*;
use crossbeam_channel::Receiver;
use serde::{Deserialize, Serialize};
use surrealdb::{
    engine::remote::ws::{Client, Ws},
    opt::auth::Root,
    sql::Thing,
    Surreal,
};

pub struct SurrealClientPlugin;

impl Plugin for SurrealClientPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<SurrealTasks>()
            .register_type::<SurrealClient>()
            .add_system(client_connector);
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

#[derive(Resource, Deref, DerefMut, Default)]
struct SurrealTasks(Vec<SurrealTask>);

#[derive(Component)]
enum SurrealTask {
    Client(Receiver<Result<SurrealClient, surrealdb::Error>>),
    Action(Receiver<()>),
}

fn client_connector(
    mut connect: Local<bool>,
    mut commands: Commands,
    mut surreal_tasks: ResMut<SurrealTasks>,
) {
    if !*connect {
        *connect = true;

        let connect = queue_task(async move {
            let address = "127.0.0.1:8000";
            info!("Connecting to SurrealDB: {}", address);
            let connect = Surreal::new::<Ws>(address).await;
            match connect {
                Ok(client) => {
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

                    Ok(client_resource)
                }
                Err(e) => {
                    error!("Failed to connect to SurrealDB: {:?}", e);
                    Err(e)
                }
            }
        });

        surreal_tasks.push(SurrealTask::Client(connect));
    }

    let mut removes = vec![];
    let mut adds = vec![];
    for (idx, task) in surreal_tasks.iter().enumerate() {
        match task {
            SurrealTask::Client(receiver) => {
                if let Ok(client) = receiver.try_recv() {
                    removes.push(idx);
                    info!("Surreal client received");
                    if let Ok(client) = client {
                        info!("Surreal client received");
                        commands.insert_resource(client.clone());
                        if let Some(client) = client.client {
                            let task = queue_task(async move {
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
                            adds.push(SurrealTask::Action(task));
                        }
                    }
                }
            }
            SurrealTask::Action(receiver) => {
                if let Ok(_) = receiver.try_recv() {
                    removes.push(idx);
                    info!("Surreal action received");
                }
            }
        }
    }

    // remove tasks by index
    removes.sort_by(|a, b| b.cmp(a));
    for remove in removes {
        surreal_tasks.remove(remove);
    }

    // add tasks
    for add in adds {
        surreal_tasks.push(add);
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
