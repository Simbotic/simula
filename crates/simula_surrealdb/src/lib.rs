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
        app.add_startup_system(setup).add_system(run);
    }
}

#[derive(Resource, Deref)]
struct SurrealClient(Surreal<Client>);

#[derive(Resource, Deref)]
struct SurrealClientReceiver(Receiver<Surreal<Client>>);

fn setup(mut commands: Commands) {
    let (sender, receiver) = bounded(1);
    commands.insert_resource(SurrealClientReceiver(receiver));

    let thread_pool = AsyncComputeTaskPool::get();
    let _ = thread_pool.spawn(async move {
        if let Ok(client) = Surreal::new::<Ws>("127.0.0.1:8000").await {
            info!("Connected to SurrealDB");

            let _ = client
                .signin(Root {
                    username: "root",
                    password: "root",
                })
                .await;
            let _ = client.use_ns("default").use_db("default").await;

            sender.send(client).unwrap();
        };
    });
}

fn run(mut commands: Commands, surreal_client_receiver: Res<SurrealClientReceiver>) {
    if let Some(client) = surreal_client_receiver.try_recv().ok() {
        info!("Surreal client received");

        commands.insert_resource(SurrealClient(client.clone()));

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
