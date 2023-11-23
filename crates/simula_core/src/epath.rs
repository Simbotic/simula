use bevy::{ecs::system::SystemParam, prelude::*};
use serde::de::{self, Visitor};
use serde::{Deserialize, Deserializer, Serialize, Serializer};
use std::borrow::Cow;
use std::fmt;
use std::num::ParseIntError;
use std::str::FromStr;

// "/" is parsed as E::Root
// "^" is parsed as E::First
// "$" is parsed as E::Last
// Text surrounded by square brackets (e.g. "[3]") is parsed as E::Nth with the enclosed number.
// Any other text is parsed as E::Name.

// TODO: is there a library that does this already?

#[derive(Reflect, Clone, Debug, Hash, PartialEq, Eq, Default)]
pub enum E {
    #[default]
    Root,
    Name(Cow<'static, str>),
    First,
    Last,
    Nth(usize),
    // TODO: Parent, to go up the tree
    // TODO: Regex or Predicate, to match multiple names
    // TODO: Contains, StartsWith, EndsWith, to match multiple names
    // TODO: Wildcard child, to keep going down the tree
}

#[derive(Deref, DerefMut, Reflect, Clone, Debug, Hash, PartialEq, Eq, Default)]
pub struct EPath(Vec<E>);

impl ToString for EPath {
    fn to_string(&self) -> String {
        let mut s = String::new();
        let mut prefix_forward_slash = false;
        for e in &self.0 {
            if prefix_forward_slash {
                s.push_str("/");
            }
            match e {
                E::Root => {
                    s.push_str("/");
                }
                E::Name(name) => {
                    s.push_str(name);
                    prefix_forward_slash = true;
                }
                E::First => {
                    s.push_str("^");
                    prefix_forward_slash = true;
                }
                E::Last => {
                    s.push_str("$");
                    prefix_forward_slash = true;
                }
                E::Nth(n) => {
                    s.push_str(&format!("[{}]", n));
                    prefix_forward_slash = true;
                }
            }
        }
        trace!("EPath::to_string: {:?} {:?}", self, s);
        s
    }
}

#[derive(Debug)]
pub enum ParseEPathError {
    UnknownKeyword,
    ParseIntError(ParseIntError),
}

impl std::fmt::Display for ParseEPathError {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(f, "ParseEPathError occurred")
    }
}

impl From<ParseIntError> for ParseEPathError {
    fn from(err: ParseIntError) -> Self {
        ParseEPathError::ParseIntError(err)
    }
}

impl FromStr for E {
    type Err = ParseEPathError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "/" => panic!("Root should've been handled by EPath::from_str"),
            "^" => Ok(E::First),
            "$" => Ok(E::Last),
            _ if s.starts_with("[") && s.ends_with("]") => {
                let number_part = &s[1..s.len() - 1];
                match number_part.parse::<usize>() {
                    Ok(n) => Ok(E::Nth(n)),
                    Err(e) => Err(e.into()),
                }
            }
            _ => Ok(E::Name(s.to_owned().into())),
        }
    }
}

impl FromStr for EPath {
    type Err = ParseEPathError;

    fn from_str(s: &str) -> Result<Self, Self::Err>
    where
        Self::Err: std::fmt::Debug,
    {
        let parts: Vec<&str> = s.split('/').collect();
        let mut result = vec![];

        // Handle leading root
        if s.starts_with("/") {
            result.push(E::Root);
        }

        for part in parts {
            if part.is_empty() {
                continue;
            }
            let e: E = part.parse()?;
            result.push(e);
        }
        Ok(EPath(result))
    }
}

impl TryFrom<String> for EPath {
    type Error = ParseEPathError;

    fn try_from(s: String) -> Result<Self, Self::Error> {
        s.parse::<EPath>()
    }
}

impl Serialize for EPath {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        let s = self.to_string();
        serializer.serialize_str(&s)
    }
}

struct EPathVisitor;

impl<'de> Visitor<'de> for EPathVisitor {
    type Value = EPath;

    fn expecting(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
        formatter.write_str("a string in EPath format")
    }

    fn visit_str<E>(self, value: &str) -> Result<EPath, E>
    where
        E: de::Error,
    {
        value.parse::<EPath>().map_err(E::custom)
    }
}

impl<'de> Deserialize<'de> for EPath {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        deserializer.deserialize_str(EPathVisitor)
    }
}

#[derive(SystemParam)]
pub struct EPathQueries<'w, 's> {
    pub names: Query<'w, 's, &'static Name>,
    pub parents: Query<'w, 's, &'static Parent>,
    pub children: Query<'w, 's, &'static Children>,
    pub roots: Query<'w, 's, Entity, Without<Parent>>,
}

#[derive(Reflect, Clone, Debug, Hash, PartialEq, Eq, Serialize, Deserialize)]
pub struct EEntity {
    pub path: EPath,
    pub entity: Entity,
    pub name: Option<Cow<'static, str>>,
}

pub fn select(parent: Option<Entity>, path: &[E], equeries: &EPathQueries) -> Vec<EEntity> {
    let mut entities = Vec::new();
    let mut breadcrumb = EPath::default();
    select_traverse(&mut entities, &mut breadcrumb, parent, &path, equeries);
    entities
}

fn select_traverse(
    entities: &mut Vec<EEntity>,
    breadcrumb: &mut EPath,
    parent: Option<Entity>,
    path: &[E],
    equeries: &EPathQueries,
) {
    let item = path.first();

    match item {
        Some(E::Root) => {
            breadcrumb.push(E::Root);

            if !parent.is_none() {
                panic!("Root must be first element in path");
            }
            select_traverse(entities, breadcrumb, None, &path[1..], equeries);
        }
        Some(E::Name(ename)) => {
            breadcrumb.push(E::Name(ename.clone()));

            if let Some(ancestor) = parent {
                if let Ok(childs) = equeries.children.get(ancestor) {
                    for child in childs.iter() {
                        if let Ok(name) = equeries.names.get(*child) {
                            if ename == name.as_ref() {
                                if path.len() == 1 {
                                    entities.push(EEntity {
                                        path: breadcrumb.clone(),
                                        entity: *child,
                                        name: Some(name.as_ref().to_owned().into()),
                                    });
                                } else {
                                    select_traverse(
                                        entities,
                                        breadcrumb,
                                        Some(*child),
                                        &path[1..],
                                        equeries,
                                    );
                                }
                            }
                        }
                    }
                }
            } else {
                for root in equeries.roots.iter() {
                    if let Ok(name) = equeries.names.get(root) {
                        if ename == name.as_ref() {
                            if path.len() == 1 {
                                entities.push(EEntity {
                                    path: breadcrumb.clone(),
                                    entity: root,
                                    name: Some(name.as_ref().to_owned().into()),
                                });
                            } else {
                                select_traverse(
                                    entities,
                                    breadcrumb,
                                    Some(root),
                                    &path[1..],
                                    equeries,
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
                if let Ok(childs) = equeries.children.get(ancestor) {
                    if let Some(child) = childs.iter().next() {
                        if path.len() == 1 {
                            let name = equeries
                                .names
                                .get(*child)
                                .and_then(|name| Ok(name.as_ref().to_owned().into()))
                                .ok();
                            entities.push(EEntity {
                                path: breadcrumb.clone(),
                                entity: *child,
                                name,
                            });
                        } else {
                            select_traverse(
                                entities,
                                breadcrumb,
                                Some(*child),
                                &path[1..],
                                equeries,
                            );
                        }
                    }
                }
            } else {
                if let Some(root) = equeries.roots.iter().next() {
                    if path.len() == 1 {
                        let name = equeries
                            .names
                            .get(root)
                            .and_then(|name| Ok(name.as_ref().to_owned().into()))
                            .ok();
                        entities.push(EEntity {
                            path: breadcrumb.clone(),
                            entity: root,
                            name,
                        });
                    } else {
                        select_traverse(entities, breadcrumb, Some(root), &path[1..], equeries);
                    }
                }
            }
        }
        Some(E::Last) => {
            breadcrumb.push(E::Last);

            if let Some(ancestor) = parent {
                if let Ok(childs) = equeries.children.get(ancestor) {
                    if let Some(child) = childs.iter().last() {
                        if path.len() == 1 {
                            let name = equeries
                                .names
                                .get(*child)
                                .and_then(|name| Ok(name.as_ref().to_owned().into()))
                                .ok();
                            entities.push(EEntity {
                                path: breadcrumb.clone(),
                                entity: *child,
                                name,
                            });
                        } else {
                            select_traverse(
                                entities,
                                breadcrumb,
                                Some(*child),
                                &path[1..],
                                equeries,
                            );
                        }
                    }
                }
            } else {
                if let Some(root) = equeries.roots.iter().last() {
                    if path.len() == 1 {
                        let name = equeries
                            .names
                            .get(root)
                            .and_then(|name| Ok(name.as_ref().to_owned().into()))
                            .ok();
                        entities.push(EEntity {
                            path: breadcrumb.clone(),
                            entity: root,
                            name,
                        });
                    } else {
                        select_traverse(entities, breadcrumb, Some(root), &path[1..], equeries);
                    }
                }
            }
        }
        Some(E::Nth(index)) => {
            breadcrumb.push(E::Nth(*index));

            if let Some(ancestor) = parent {
                if let Ok(childs) = equeries.children.get(ancestor) {
                    if let Some(child) = childs.iter().nth(*index) {
                        if path.len() == 1 {
                            let name = equeries
                                .names
                                .get(*child)
                                .and_then(|name| Ok(name.as_ref().to_owned().into()))
                                .ok();
                            entities.push(EEntity {
                                path: breadcrumb.clone(),
                                entity: *child,
                                name,
                            });
                        } else {
                            select_traverse(
                                entities,
                                breadcrumb,
                                Some(*child),
                                &path[1..],
                                equeries,
                            );
                        }
                    }
                }
            } else {
                if let Some(root) = equeries.roots.iter().nth(*index) {
                    if path.len() == 1 {
                        let name = equeries
                            .names
                            .get(root)
                            .and_then(|name| Ok(name.as_ref().to_owned().into()))
                            .ok();
                        entities.push(EEntity {
                            path: breadcrumb.clone(),
                            entity: root,
                            name,
                        });
                    } else {
                        select_traverse(entities, breadcrumb, Some(root), &path[1..], equeries);
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
        equeries: EPathQueries,
    ) {
        // Entity names and IDs
        // for (entity, name) in &debug {
        //     println!("{:?}: {}", entity, name.as_ref());
        // }

        let entities = select(path_test.parent, &path_test.path, &equeries);

        // let result = ron::to_string(&entities).unwrap();
        assert_eq!(path_test.result, entities);
    }

    #[test]
    fn test_epath_root_name() {
        // Create app
        let mut app = App::new();

        // Add testor
        app.add_systems(Update, select_test);

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
        app.add_systems(Update, select_test);

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
        app.add_systems(Update, select_test);

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
        app.add_systems(Update, select_test);

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
        app.add_systems(Update, select_test);

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

    #[test]
    fn test_epath_from_string_test_root_1_first_first_first_first() {
        // Create app
        let mut app = App::new();

        // Add testor
        app.add_systems(Update, select_test);

        // Command queue
        let mut command_queue = CommandQueue::default();
        let mut commands = Commands::new(&mut command_queue, &app.world);

        // Spawn hierarchy
        spawn_hierarchy(&mut commands);

        // Selector
        let path = EPath::from_str("/[1]/^/^/^/^").unwrap();
        let path = path.to_string();
        let path = EPath::from_str(&path).unwrap();

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
    fn test_epath_from_string_test_relative_first_name() {
        // Create app
        let mut app = App::new();

        // Add testor
        app.add_systems(Update, select_test);

        // Command queue
        let mut command_queue = CommandQueue::default();
        let mut commands = Commands::new(&mut command_queue, &app.world);

        // Spawn hierarchy
        spawn_hierarchy(&mut commands);

        // Selector
        let path = EPath::from_str("^/Pul").unwrap();
        let path = path.to_string();
        let path = EPath::from_str(&path).unwrap();

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
