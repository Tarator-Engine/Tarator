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

use erased_serde::Serialize as ESerialize;
use fxhash::FxBuildHasher;
use serde::ser::{SerializeMap, SerializeSeq, SerializeStruct};
use std::collections::HashMap;

use crate::component::Info;
use super::SerdeComponent;
use tar_ecs::prelude::*;


/// A wrapper for world serialization
///
/// # Example
///
/// ```
/// use tar_ecs::prelude::*;
/// use serde::{Serialize, Deserialize};
/// use scr_types::{
///     Component,
///     component::{
///         Info,
///         ser::{SerializeCallback, SerWorld},
///         de::DeWorld
///     }
/// };
///
/// #[derive(Component, Serialize, Deserialize)]
/// struct Foo(u32);
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
/// serde_json::to_string(&SerWorld::new(&world, entity_id)).unwrap();
/// ```
pub struct SerWorld<'a> {
    world: &'a World,
    id: uuid::Uuid,
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



/// Serializes all the components existing on an entity
#[derive(Callback, Default)]
pub struct SerializeCallback {
    s: HashMap<&'static str, *const dyn ESerialize, FxBuildHasher>,
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
    entity: Entity,
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
    world: &'a World,
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

