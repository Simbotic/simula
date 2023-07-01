use bevy::prelude::*;
use serde::{Deserialize, Serialize};
use std::borrow::Cow;

#[derive(
    Reflect, FromReflect, Clone, Debug, Hash, PartialEq, Eq, Default, Serialize, Deserialize,
)]
pub enum E {
    #[default]
    Root,
    Name(Cow<'static, str>),
    First,
    Last,
    Nth(usize),
    // TODO: Parent, to go up the tree
    // TODO: Regex, to match multiple names
    // TODO: Contains, StartsWith, EndsWith, to match multiple names
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

pub fn select(
    entities: &mut Vec<EEntity>,
    breadcrumb: &mut EPath,
    parent: Option<Entity>,
    path: &[E],
    names: &Query<&Name>,
    parents: &Query<&Parent>,
    children: &Query<&Children>,
    roots: &Query<Entity, Without<Parent>>,
) {
    let item = path.first();

    match item {
        Some(E::Root) => {
            breadcrumb.push(E::Root);

            if !parent.is_none() {
                panic!("Root must be first element in path");
            }
            select(
                entities,
                breadcrumb,
                None,
                &path[1..],
                names,
                parents,
                children,
                roots,
            );
        }
        Some(E::Name(ename)) => {
            breadcrumb.push(E::Name(ename.clone()));

            if let Some(ancestor) = parent {
                if let Ok(childs) = children.get(ancestor) {
                    for child in childs.iter() {
                        if let Ok(name) = names.get(*child) {
                            if ename == name.as_ref() {
                                if path.len() == 1 {
                                    entities.push(EEntity {
                                        path: breadcrumb.clone(),
                                        entity: *child,
                                        name: Some(name.as_ref().to_owned().into()),
                                    });
                                } else {
                                    select(
                                        entities,
                                        breadcrumb,
                                        Some(*child),
                                        &path[1..],
                                        names,
                                        parents,
                                        children,
                                        roots,
                                    );
                                }
                            }
                        }
                    }
                }
            } else {
                for root in roots.iter() {
                    if let Ok(name) = names.get(root) {
                        if ename == name.as_ref() {
                            if path.len() == 1 {
                                entities.push(EEntity {
                                    path: breadcrumb.clone(),
                                    entity: root,
                                    name: Some(name.as_ref().to_owned().into()),
                                });
                            } else {
                                select(
                                    entities,
                                    breadcrumb,
                                    Some(root),
                                    &path[1..],
                                    names,
                                    parents,
                                    children,
                                    roots,
                                );
                            }
                        }
                    }
                }
            }
        }
        Some(E::First) => {
            breadcrumb.push(E::First);

            if let Some(ancestor) = parent {
                if let Ok(childs) = children.get(ancestor) {
                    if let Some(child) = childs.iter().next() {
                        if path.len() == 1 {
                            let name = names
                                .get(*child)
                                .and_then(|name| Ok(name.as_ref().to_owned().into()))
                                .ok();
                            entities.push(EEntity {
                                path: breadcrumb.clone(),
                                entity: *child,
                                name,
                            });
                        } else {
                            select(
                                entities,
                                breadcrumb,
                                Some(*child),
                                &path[1..],
                                names,
                                parents,
                                children,
                                roots,
                            );
                        }
                    }
                }
            } else {
                if let Some(root) = roots.iter().next() {
                    if path.len() == 1 {
                        let name = names
                            .get(root)
                            .and_then(|name| Ok(name.as_ref().to_owned().into()))
                            .ok();
                        entities.push(EEntity {
                            path: breadcrumb.clone(),
                            entity: root,
                            name,
                        });
                    } else {
                        select(
                            entities,
                            breadcrumb,
                            Some(root),
                            &path[1..],
                            names,
                            parents,
                            children,
                            roots,
                        );
                    }
                }
            }
        }
        Some(E::Last) => {
            breadcrumb.push(E::Last);

            if let Some(ancestor) = parent {
                if let Ok(childs) = children.get(ancestor) {
                    if let Some(child) = childs.iter().last() {
                        if path.len() == 1 {
                            let name = names
                                .get(*child)
                                .and_then(|name| Ok(name.as_ref().to_owned().into()))
                                .ok();
                            entities.push(EEntity {
                                path: breadcrumb.clone(),
                                entity: *child,
                                name,
                            });
                        } else {
                            select(
                                entities,
                                breadcrumb,
                                Some(*child),
                                &path[1..],
                                names,
                                parents,
                                children,
                                roots,
                            );
                        }
                    }
                }
            } else {
                if let Some(root) = roots.iter().last() {
                    if path.len() == 1 {
                        let name = names
                            .get(root)
                            .and_then(|name| Ok(name.as_ref().to_owned().into()))
                            .ok();
                        entities.push(EEntity {
                            path: breadcrumb.clone(),
                            entity: root,
                            name,
                        });
                    } else {
                        select(
                            entities,
                            breadcrumb,
                            Some(root),
                            &path[1..],
                            names,
                            parents,
                            children,
                            roots,
                        );
                    }
                }
            }
        }
        Some(E::Nth(index)) => {
            breadcrumb.push(E::Nth(*index));

            if let Some(ancestor) = parent {
                if let Ok(childs) = children.get(ancestor) {
                    if let Some(child) = childs.iter().nth(*index) {
                        if path.len() == 1 {
                            let name = names
                                .get(*child)
                                .and_then(|name| Ok(name.as_ref().to_owned().into()))
                                .ok();
                            entities.push(EEntity {
                                path: breadcrumb.clone(),
                                entity: *child,
                                name,
                            });
                        } else {
                            select(
                                entities,
                                breadcrumb,
                                Some(*child),
                                &path[1..],
                                names,
                                parents,
                                children,
                                roots,
                            );
                        }
                    }
                }
            } else {
                if let Some(root) = roots.iter().nth(*index) {
                    if path.len() == 1 {
                        let name = names
                            .get(root)
                            .and_then(|name| Ok(name.as_ref().to_owned().into()))
                            .ok();
                        entities.push(EEntity {
                            path: breadcrumb.clone(),
                            entity: root,
                            name,
                        });
                    } else {
                        select(
                            entities,
                            breadcrumb,
                            Some(root),
                            &path[1..],
                            names,
                            parents,
                            children,
                            roots,
                        );
                    }
                }
            }
        }
        None => {}
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use bevy::ecs::system::CommandQueue;

    // Spawn entity hierarchy to select on
    fn spawn_hierarchy(commands: &mut Commands) {
        //  Entities created here:
        //  ID    Name
        //
        //  0v0   Foo
        //  1v0   Bar
        //  2v0   Baz
        //  3v0   Kor
        //  4v0   Qux
        //  5v0   Pep
        //  6v0   Pap
        //  7v0   Pip
        //  9v0   Pul
        //  8v0   (No Name)
        //  10v0  Pop
        //  11v0  Hok

        commands.spawn(Name::new("Foo")).with_children(|parent| {
            parent.spawn(Name::new("Bar")).with_children(|parent| {
                parent.spawn(Name::new("Baz"));
                parent.spawn(Name::new("Kor"));
            });
            parent.spawn(Name::new("Qux"));
        });
        commands.spawn(Name::new("Pep")).with_children(|parent| {
            parent.spawn(Name::new("Pap")).with_children(|parent| {
                parent.spawn(Name::new("Pip")).with_children(|parent| {
                    parent.spawn_empty().with_children(|parent| {
                        parent.spawn(Name::new("Pul"));
                    });
                });
                parent.spawn(Name::new("Pop"));
            });
            parent.spawn(Name::new("Hok"));
        });
    }

    #[derive(Resource)]
    struct EPathTest {
        parent: Option<Entity>,
        path: EPath,
        result: Vec<EEntity>,
    }

    fn select_test(
        path_test: Res<EPathTest>,
        _debug: Query<(Entity, &Name)>,
        names: Query<&Name>,
        parents: Query<&Parent>,
        childrens: Query<&Children>,
        roots: Query<Entity, Without<Parent>>,
    ) {
        // Entity names and IDs
        // for (entity, name) in &debug {
        //     println!("{:?}: {}", entity, name.as_ref());
        // }

        let mut entities = Vec::new();
        let mut breadcrumb = EPath::default();
        select(
            &mut entities,
            &mut breadcrumb,
            path_test.parent,
            &path_test.path,
            &names,
            &parents,
            &childrens,
            &roots,
        );

        // let result = ron::to_string(&entities).unwrap();
        assert_eq!(path_test.result, entities);
    }

    #[test]
    fn test_epath_root_name() {
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
        let path = EPath(vec![E::Root, E::Name("Foo".into())]);

        // Expected result
        let result = vec![EEntity {
            path: EPath(vec![E::Root, E::Name("Foo".into())]),
            entity: Entity::from_raw(0),
            name: Some("Foo".into()),
        }];
        app.insert_resource(EPathTest {
            parent: None,
            path,
            result,
        });

        // Apply commands
        command_queue.apply(&mut app.world);

        // Run systems
        app.update();
    }

    #[test]
    fn test_epath_root_first_first_last() {
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

        // Expected result
        let result = vec![EEntity {
            path: EPath(vec![E::Root, E::First, E::First, E::Last]),
            entity: Entity::from_raw(3),
            name: Some("Kor".into()),
        }];
        app.insert_resource(EPathTest {
            parent: None,
            path,
            result,
        });

        // Apply commands
        command_queue.apply(&mut app.world);

        // Run systems
        app.update();
    }

    #[test]
    fn test_epath_test_root_name_name_name() {
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
        let path = EPath(vec![
            E::Root,
            E::Name("Foo".into()),
            E::Name("Bar".into()),
            E::Name("Kor".into()),
        ]);

        // Expected result
        let result = vec![EEntity {
            path: EPath(vec![
                E::Root,
                E::Name("Foo".into()),
                E::Name("Bar".into()),
                E::Name("Kor".into()),
            ]),
            entity: Entity::from_raw(3),
            name: Some("Kor".into()),
        }];
        app.insert_resource(EPathTest {
            parent: None,
            path,
            result,
        });

        // Apply commands
        command_queue.apply(&mut app.world);

        // Run systems
        app.update();
    }

    #[test]
    fn test_epath_test_root_1_first_first_first_first() {
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
        let path = EPath(vec![
            E::Root,
            E::Nth(1),
            E::First,
            E::First,
            E::First,
            E::First,
        ]);

        // Expected result
        let result = vec![EEntity {
            path: EPath(vec![
                E::Root,
                E::Nth(1),
                E::First,
                E::First,
                E::First,
                E::First,
            ]),
            entity: Entity::from_raw(9),
            name: Some("Pul".into()),
        }];
        app.insert_resource(EPathTest {
            parent: None,
            path,
            result,
        });

        // Apply commands
        command_queue.apply(&mut app.world);

        // Run systems
        app.update();
    }

    #[test]
    fn test_epath_test_relative_first_name() {
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
        let path = EPath(vec![E::First, E::Name("Pul".into())]);

        // Expected result
        let result = vec![EEntity {
            path: EPath(vec![E::First, E::Name("Pul".into())]),
            entity: Entity::from_raw(9),
            name: Some("Pul".into()),
        }];
        app.insert_resource(EPathTest {
            parent: Some(Entity::from_raw(7)), // Parent is "Pip"
            path,
            result,
        });

        // Apply commands
        command_queue.apply(&mut app.world);

        // Run systems
        app.update();
    }
}
