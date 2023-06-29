use bevy::{ecs::query::WorldQuery, prelude::*};
use serde::{Deserialize, Serialize};
use std::borrow::Cow;

#[derive(
    Reflect, FromReflect, Clone, Debug, Hash, PartialEq, Eq, Default, Serialize, Deserialize,
)]
pub enum E {
    #[default]
    Root,
    Name(Cow<'static, str>),
    Any,
    Parent,
    First,
    Last,
    Index(usize),
}

#[derive(
    Deref,
    DerefMut,
    Reflect,
    FromReflect,
    Clone,
    Debug,
    Hash,
    PartialEq,
    Eq,
    Default,
    Serialize,
    Deserialize,
)]
pub struct EPath(Vec<E>);

#[derive(Reflect, FromReflect, Clone, Debug, Hash, PartialEq, Eq, Serialize, Deserialize)]
pub struct EEntity {
    path: EPath,
    entity: Entity,
    name: Option<Cow<'static, str>>,
}

#[derive(WorldQuery)]
pub struct EPathQuery {
    entity: Entity,
    name: Option<&'static Name>,
    parent: Option<&'static Parent>,
    children: Option<&'static Children>,
}

pub fn select(
    _entities: &mut Vec<EEntity>,
    _breadcrumb: &mut EPath,
    _path: &EPath,
    _ancestor: Option<Entity>,
    _query: &Query<EPathQuery>,
) {
    // match path.first() {
    //     Some(E::Root) => {
    //         if !ancestor.is_none() {
    //             panic!("Root must be first element in path");
    //         }
    //         breadcrumb.push(E::Root);
    //         select(entities, breadcrumb, &EPath(path[1..].into()), None, query);
    //     }
    //     Some(E::Name(name)) => {
    //         breadcrumb.push(E::Name(name.clone()));

    //         for EPathQueryItem {entity, name, parent, children} in query {
    //             // if *parent == ancestor {

    //             // }
    //         }

    //         select(entities, breadcrumb, &EPath(path[1..].into()), None, query);
    //     }
    //     Some(E::Any) => {
    //         breadcrumb.push(E::Any);
    //         select(entities, breadcrumb, &EPath(path[1..].into()), None, query);
    //     }
    //     Some(E::Parent) => {
    //         breadcrumb.push(E::Parent);
    //         select(entities, breadcrumb, &EPath(path[1..].into()), None, query);
    //     }
    //     Some(E::First) => {
    //         breadcrumb.push(E::First);
    //         select(entities, breadcrumb, &EPath(path[1..].into()), None, query);
    //     }
    //     Some(E::Last) => {
    //         breadcrumb.push(E::Last);
    //         select(entities, breadcrumb, &EPath(path[1..].into()), None, query);
    //     }
    //     Some(E::Index(index)) => {
    //         breadcrumb.push(E::Index(*index));
    //         select(entities, breadcrumb, &EPath(path[1..].into()), None, query);
    //     }
    //     None => {
    //         if let Some(entity) = parent {
    //             entities.push(EEntity {
    //                 path: breadcrumb.clone(),
    //                 entity,
    //                 name: query.get_component::<Name>(entity).map(|name| name.0.clone()),
    //             });
    //         }
    //     }
    // }
}

#[cfg(test)]
mod tests {
    use super::*;
    use bevy::ecs::system::CommandQueue;

    // Spawn entity hierarchy to select on
    fn spawn_hierarchy(commands: &mut Commands) {
        commands.spawn(Name::new("Foo")).with_children(|parent| {
            parent.spawn(Name::new("Bar")).with_children(|parent| {
                parent.spawn(Name::new("Baz"));
                parent.spawn(Name::new("Kor"));
            });
            parent.spawn(Name::new("Qux"));
        });
        commands.spawn(Name::new("Pep")).with_children(|parent| {
            parent.spawn(Name::new("Pap")).with_children(|parent| {
                parent.spawn(Name::new("Pip"));
                parent.spawn(Name::new("Pop"));
            });
            parent.spawn(Name::new("Pup"));
        });
    }

    #[derive(Resource)]
    struct EPathTest {
        path: EPath,
        result: String,
    }

    fn select_test(path_test: Res<EPathTest>, query: Query<EPathQuery>) {
        let mut entities = Vec::new();
        let mut breadcrumb = EPath::default();
        select(
            &mut entities,
            &mut breadcrumb,
            &path_test.path,
            None,
            &query,
        );
        let result = ron::to_string(&entities).unwrap();
        assert_eq!(path_test.result, result);
    }

    #[test]
    fn test_root_first_first_last() {
        // Create app
        let mut app = App::new();

        // Add testor
        app.add_system(select_test);

        // Command queue
        let mut command_queue = CommandQueue::default();
        let mut commands = Commands::new(&mut command_queue, &app.world);

        // Spawn hierarchy
        spawn_hierarchy(&mut commands);

        // Selector
        let path = EPath(vec![E::Root, E::First, E::First, E::Last]);
        let result = "[]".to_string();
        app.insert_resource(EPathTest { path, result });

        // Apply commands
        command_queue.apply(&mut app.world);

        // Run systems
        app.update();
    }
}
