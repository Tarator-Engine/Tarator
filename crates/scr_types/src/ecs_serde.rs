//! The structure of the serializer is following:
//! 
//! World:
//! | id: Uuid
//! | ee:
//! | | entity[]:
//! | | | name: String
//! | | | id: Uuid
//! | | | components[]:
//! | | | | name > component
//!
//! An example of a (de)serialization of a world can be found in `tests::serdeialize`] at the
//! bottom of this file.

use std::collections::HashMap;
use fxhash::FxBuildHasher;
use serde::ser::{ SerializeStruct, SerializeMap, SerializeSeq };
use erased_serde::{
    Serialize as ESerialize,
    Deserializer as EDeserializer
};

use tar_ecs::prelude::*;
use crate::components::Info;


/// To be implemented on Components that want to be serde-ed
///
/// # Example
///
/// ```
/// use serde::{Serialize, Deserialize};
/// use tar_ecs::prelude::*;
///
/// #[derive(Component, Serialize, Deserialize)]
/// struct Foo {
///     bar: u32
/// }
///
/// impl SerdeComponent for Foo {
///     const NAME: &'static str = "example::Foo";
/// }
/// ```
pub trait SerdeComponent: Component + serde::Serialize + for<'a> serde::Deserialize<'a> {
    const NAME: &'static str;

    fn construct(deserializer: &mut dyn EDeserializer, world: &mut World, entity: Entity) -> Result<(), erased_serde::Error> {
        let this: Self = erased_serde::deserialize(deserializer)?; // TODO: Use unwrap_or_default instad?
        world.entity_set(entity, this);

        Ok(())
    }
}


/// A wrapper for world serialization
///
/// # Example
///
/// ```
/// use tar_ecs::prelude::*;
/// 
/// #[derive(Component, Serialize, Deserialize)]
/// struct Foo(u32);
///
/// impl SerdeComponent for Foo {
///     const NAME: &'static str = "example::Foo";
/// }
///
/// let mut world = World::new();
/// world.component_add_callback::<SerializeCallback, Foo>();
/// let entity = world.entity_create();
/// world.entity_set(entity, Foo(20));
///
/// // Entity needs Info component to be serialized
/// let entity_id = uuid::Uuid::new_v4();
/// world.entity_set(entity, Info { id: entity_id, name: "Entity".into() });
///
/// let world_id = uuid::Uuid::new_v4();
/// serde_json::to_string(SerWorld::new(&world, entity_id)).unwrap();
/// ```
pub struct SerWorld<'a> {
    world: &'a World,
    id: uuid::Uuid
}

impl<'a> SerWorld<'a> {
    pub fn new(world: &'a World, id: uuid::Uuid) -> Self {
        Self { world, id }
    }
}

impl<'a> serde::Serialize for SerWorld<'a> {
    fn serialize<S: serde::Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        let mut s = serializer.serialize_struct("World", 2)?;
        let entity_entries = SerEntityEntry::new(self.world);

        s.serialize_field("id", &self.id.to_string())?;
        s.serialize_field("ee", &entity_entries)?;

        s.end()
    }
}


pub struct DeWorld {
    pub id: uuid::Uuid,
    pub world: World
}

#[derive(Default)]
pub struct DeWorldBuilder<'a> {
    constuctors: ConstructorMap<'a>
}

impl<'a> DeWorldBuilder<'a> {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn constructor<T: SerdeComponent>(mut self) -> Self {
        self.constuctors.insert(T::NAME, T::construct);
        self
    }

    pub fn build<'de, D: serde::Deserializer<'de>>(self, deserializer: D) -> Result<DeWorld, D::Error> {
        use serde::de::DeserializeSeed;

        self.deserialize(deserializer)
    }
}


impl<'a, 'de> serde::de::DeserializeSeed<'de> for DeWorldBuilder<'a> {
    type Value = DeWorld;

    fn deserialize<D: serde::Deserializer<'de>>(self, deserializer: D) -> Result<Self::Value, D::Error> {
        deserializer.deserialize_struct("World", &["id", "ee"], DeWorldVisitor { constuctors: self.constuctors }) 
    }
}

/// Serializes all the components existing on an entity
#[derive(Callback, Default)]
struct SerializeCallback {
    s: HashMap<&'static str, *const dyn ESerialize, FxBuildHasher>
}


impl<T: SerdeComponent> Callback<T> for SerializeCallback {
    fn callback(&mut self, component: &T) {
        self.s.insert(T::NAME, component);
    }
}


struct SerComponent<'a> {
    world: &'a World,
    entity: Entity,
}

impl<'a> SerComponent<'a> {
    pub fn new(world: &'a World, entity: Entity) -> Self {
        Self { world, entity }
    }
}

impl<'a> serde::Serialize for SerComponent<'a> {
    fn serialize<S: serde::Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        let mut callback = SerializeCallback::default();
        self.world.entity_callback(self.entity, &mut callback);

        let mut s = serializer.serialize_map(Some(callback.s.len()))?;
        
        for (key, value) in callback.s {
            s.serialize_entry(key, unsafe { &*value })?
        }

        s.end()
    }
}


struct SerEntity<'a> {
    world: &'a World,
    entity: Entity
}

impl<'a> SerEntity<'a> {
    pub fn new(world: &'a World, entity: Entity) -> Self {
        Self { world, entity }
    }
}

impl<'a> serde::Serialize for SerEntity<'a> {
    fn serialize<S: serde::Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        let mut s = serializer.serialize_struct("Entity", 3)?;
    
        let info = self.world.entity_get::<Info>(self.entity).unwrap();
        let components = SerComponent::new(self.world, self.entity);

        s.serialize_field("name", &info.name)?;
        s.serialize_field("id", &info.id.to_string())?;
        s.serialize_field("components", &components)?;

        s.end()
    }
}


struct SerEntityEntry<'a> {
    world: &'a World
}

impl<'a> SerEntityEntry<'a> {
    pub fn new(world: &'a World) -> Self {
        Self { world }
    }
}

impl<'a> serde::Serialize for SerEntityEntry<'a> {
    fn serialize<S: serde::Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        let entities = self.world.entity_collect::<Info>();
        let mut s = serializer.serialize_seq(Some(entities.len()))?;

        for entity in entities {
            let entity_entry = SerEntity::new(self.world, entity);
            s.serialize_element(&entity_entry).unwrap(); // TODO: We don't want missing entities
        }

        s.end()
    }
}





type ConstructorFunc = fn(&mut dyn EDeserializer, &mut World, Entity) -> Result<(), erased_serde::Error>;
type ConstructorMap<'a> = HashMap<&'a str, ConstructorFunc, FxBuildHasher>;



struct DeWorldVisitor<'a> {
    constuctors: ConstructorMap<'a>
}

struct DeEntityEntry<'a> {
    constuctors: &'a ConstructorMap<'a>
}

struct DeEntityEntryVisitor<'a> {
    constuctors: &'a ConstructorMap<'a>
}

struct DeEntity<'a> {
    world: &'a mut World,
    constuctors: &'a ConstructorMap<'a>
}

struct DeEntityVisitor<'a> {
    world: &'a mut World,
    constuctors: &'a ConstructorMap<'a>
}

struct DeComponents<'a> {
    entity: Entity,
    world: &'a mut World,
    constuctors: &'a ConstructorMap<'a>
}

struct DeComponentsVisitor<'a> {
    entity: Entity,
    world: &'a mut World,
    constuctors: &'a ConstructorMap<'a>
}

struct DeComponent<'a> {
    entity: Entity,
    world: &'a mut World,
    func: ConstructorFunc
}


impl<'a, 'de> serde::de::DeserializeSeed<'de> for DeComponent<'a> {
    type Value = ();

    fn deserialize<D: serde::Deserializer<'de>>(self, deserializer: D) -> Result<Self::Value, D::Error> {
        let d = &mut <dyn erased_serde::Deserializer>::erase(deserializer);
        (self.func)(d, self.world, self.entity).map_err(|e| serde::de::Error::custom(format!("Could not parse component: {e}")))
    }
}


impl<'a, 'de> serde::de::Visitor<'de> for DeComponentsVisitor<'a> {
    type Value = ();

    fn expecting(&self, formatter: &mut std::fmt::Formatter) -> std::fmt::Result {
        formatter.write_str("Could not parse components")
    }

    fn visit_map<A: serde::de::MapAccess<'de>>(self, mut map: A) -> Result<Self::Value, A::Error> {
        while let Some(key) = map.next_key::<&str>()? {
            map.next_value_seed(
                DeComponent { 
                    entity: self.entity,
                    world: self.world,
                    func: *self.constuctors.get(key).ok_or(serde::de::Error::custom(format!("Component constructor for {key} not initialized")))?
                })?
        }

        Ok(())
    }
}


impl<'a, 'de> serde::de::DeserializeSeed<'de> for DeComponents<'a> {
    type Value = ();

    fn deserialize<D: serde::Deserializer<'de>>(self, deserializer: D) -> Result<Self::Value, D::Error> {
        deserializer.deserialize_map(DeComponentsVisitor { world: self.world, entity: self.entity, constuctors: self.constuctors }) 
    }
}


impl<'a, 'de> serde::de::Visitor<'de> for DeEntityVisitor<'a> {
    type Value = ();

    fn expecting(&self, formatter: &mut std::fmt::Formatter) -> std::fmt::Result {
        formatter.write_str("Could not parse entity")
    }

    fn visit_map<A: serde::de::MapAccess<'de>>(self, mut map: A) -> Result<Self::Value, A::Error> {
        let mut id = None;
        let mut name = None;
        let entity = self.world.entity_create();
        
        while let Some(key) = map.next_key()? {
            match key {
                "id" => id = Some(uuid::Uuid::parse_str(map.next_value()?).map_err(|e| serde::de::Error::custom(format!("Could not parse world id: {}", e)))?),
                "name" => name = Some(map.next_value()?),
                "components" => map.next_value_seed(DeComponents { world: self.world, entity, constuctors: self.constuctors })?,
                _ => ()
            }
        }

        let Some(id) = id else {
            return Err(serde::de::Error::missing_field("id"));
        };

        let Some(name) = name else {
            return Err(serde::de::Error::missing_field("name"));
        };

        self.world.entity_set(entity, Info { id, name });

        Ok(())
    }
}


impl<'a, 'de> serde::de::DeserializeSeed<'de> for DeEntity<'a> {
    type Value = ();

    fn deserialize<D: serde::Deserializer<'de>>(self, deserializer: D) -> Result<Self::Value, D::Error> {
        deserializer.deserialize_struct("Entity", &["id", "name", "components"], DeEntityVisitor { world: self.world, constuctors: self.constuctors })  
    }
}


impl<'a, 'de> serde::de::Visitor<'de> for DeEntityEntryVisitor<'a> {
    type Value = World;

    fn expecting(&self, formatter: &mut std::fmt::Formatter) -> std::fmt::Result {
        formatter.write_str("Could not parse entity entries")
    }

    fn visit_seq<A: serde::de::SeqAccess<'de>>(self, mut seq: A) -> Result<Self::Value, A::Error> {
        let mut world = World::new();

        while let Some(()) = seq.next_element_seed(DeEntity { world: &mut world, constuctors: &self.constuctors})? {}

        Ok(world)
    }
}


impl<'a, 'de> serde::de::DeserializeSeed<'de> for DeEntityEntry<'a> {
    type Value = World;

    fn deserialize<D: serde::Deserializer<'de>>(self, deserializer: D) -> Result<Self::Value, D::Error> {
        deserializer.deserialize_seq(DeEntityEntryVisitor { constuctors: self.constuctors }) 
    }
}


impl<'a, 'de> serde::de::Visitor<'de> for DeWorldVisitor<'a> {
    type Value = DeWorld;

    fn expecting(&self, formatter: &mut std::fmt::Formatter) -> std::fmt::Result {
        formatter.write_str("Could not parse world")
    }

    fn visit_seq<A: serde::de::SeqAccess<'de>>(self, mut seq: A) -> Result<Self::Value, A::Error> {
        let id = uuid::Uuid::parse_str(seq.next_element()?.ok_or(serde::de::Error::invalid_length(0, &self))?).map_err(|e| serde::de::Error::custom(format!("Could not parse world id: {}", e)))?;
        let world = seq.next_element_seed(DeEntityEntry { constuctors: &self.constuctors })?.ok_or(serde::de::Error::invalid_length(1, &self))?;

        Ok(DeWorld { id, world })
    }

    fn visit_map<A: serde::de::MapAccess<'de>>(self, mut map: A) -> Result<Self::Value, A::Error> {
        let mut id = None;
        let mut world = None;

        while let Some(key) = map.next_key()? {
            match key {
                "id" => id = Some(uuid::Uuid::parse_str(map.next_value()?).map_err(|e| serde::de::Error::custom(format!("Could not parse world id: {}", e)))?),
                "ee" => world = Some(map.next_value_seed(DeEntityEntry { constuctors: &self.constuctors })?),
                _ => ()
            }
        }

        let Some(id) = id else {
            return Err(serde::de::Error::missing_field("id"));
        };

        let Some(world) = world else {
            return Err(serde::de::Error::missing_field("ee"));
        };

        Ok(DeWorld { id, world })
    }
}





#[cfg(test)]
mod tests {
    use tar_ecs::prelude::*;    
    use super::{SerdeComponent, SerWorld, SerializeCallback, DeWorldBuilder};
    use crate::components::{Info, Transform};
    use serde::{Serialize, Deserialize};
    use serde_json::de::Deserializer as JsonDeserializer;

    #[derive(Debug, Component, Serialize, Deserialize)]
    struct Foo {
        foo1: u32,
        foo2: String
    }

    impl SerdeComponent for Foo {
        const NAME: &'static str = "test::Foo";
    }

    #[derive(Debug, Component, Serialize, Deserialize)]
    struct Bar(Vec<u32>);

    impl SerdeComponent for Bar {
        const NAME: &'static str = "test::Bar";
    }

    #[test]
    fn serdeialize() {
        let id = uuid::Uuid::new_v4();
        let serialized = {
            let mut world = World::new();
            world.component_add_callback::<SerializeCallback, Transform>();
            world.component_add_callback::<SerializeCallback, Foo>();
            world.component_add_callback::<SerializeCallback, Bar>();

            let entity = world.entity_create();
            let data = (
                Info { id, name: "GigachadEntity".into() },
                Transform::default(),
                Foo { foo1: 50, foo2: "We are baba!".into() },
                Bar(vec![50, 6665, 13407, 324])
            );
            world.entity_set(entity, data);

            serde_json::to_string(&SerWorld::new(&world, uuid::Uuid::new_v4())).unwrap()
        };

        let deworld = DeWorldBuilder::new()
            .constructor::<Transform>()
            .constructor::<Foo>()
            .constructor::<Bar>()
            .build(&mut JsonDeserializer::from_str(&serialized)).unwrap();

        deworld.world
            .component_query::<(Info, Transform, Foo, Bar)>()
            .for_each(|(info, t, foo, bar)| {
                assert!(info.name == "GigachadEntity");
                assert!(info.id == id);
                assert!(t == &Transform::default());
                assert!(foo.foo1 == 50);
                assert!(foo.foo2 == "We are baba!");
                assert!(bar.0 == vec![50, 6665, 13407, 324]);
            });
    }
}

