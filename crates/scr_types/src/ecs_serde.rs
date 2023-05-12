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

use tar_ecs::prelude::*;
use serde::ser::{ SerializeStruct, SerializeMap, SerializeSeq };
use crate::components::Info;

/// To be implemented on Components that want to be serde-ed
pub trait SerdeComponent: Component + serde::Serialize + for<'a> serde::Deserialize<'a> {
    const NAME: &'static str;
}

pub struct WorldSerializer<'a> {
    world: &'a World
}

pub struct EntityEntrySerializer<'a> {
    world: &'a World
}

pub struct EntitySerializer<'a> {
    world: &'a World,
    entity: Entity
}

pub struct ComponentSerializer<'a> {
    world: &'a World,
    entity: Entity,
}

/// Serializes all the components existing on an entity
#[derive(Callback)]
struct SerializeCallback<S: serde::Serializer> {
    s: Result<S::SerializeMap, S::Error>
}


impl<S: serde::Serializer> SerializeCallback<S> {
    pub fn new(serializer: S) -> Self {
        Self { s: serializer.serialize_map(None /* TODO: This can be calculated */) } 
    }

    pub fn end(self) -> Result<S::Ok, S::Error> {
        self.s?.end()
    }
}

impl<S: serde::Serializer, T: SerdeComponent> Callback<T> for SerializeCallback<S> {
    fn callback(&mut self, component: &T) {
        let Ok(s) = self.s.as_mut() else {
            return;
        };
        let Err(e) = s.serialize_entry(T::NAME, component) else {
            return;
        };
        self.s = Err(e);
    }
}


impl<'a> ComponentSerializer<'a> {
    pub fn new(world: &'a World, entity: Entity) -> Self {
        Self { world, entity }
    }
}

impl<'a> serde::Serialize for ComponentSerializer<'a> {
    fn serialize<S: serde::Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        let mut callback = SerializeCallback::new(serializer);
        self.world.entity_callback(self.entity, &mut callback);
        callback.end()
    }
}


impl<'a> EntitySerializer<'a> {
    pub fn new(world: &'a World, entity: Entity) -> Self {
        Self { world, entity }
    }
}

impl<'a> serde::Serialize for EntitySerializer<'a> {
    fn serialize<S: serde::Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        let mut s = serializer.serialize_struct("Entity", 3)?;

        let info = self.world.entity_get::<Info>(self.entity).unwrap();
        let components = ComponentSerializer::new(self.world, self.entity);

        s.serialize_field("name", &info.name)?;
        s.serialize_field("id", &info.id.to_string())?;
        s.serialize_field("components", &components)?;

        s.end()
    }
}


impl<'a> EntityEntrySerializer<'a> {
    pub fn new(world: &'a World) -> Self {
        Self { world }
    }
}

impl<'a> serde::Serialize for EntityEntrySerializer<'a> {
    fn serialize<S: serde::Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        let entities = self.world.entity_collect::<Info>();
        let mut s = serializer.serialize_seq(Some(entities.len()))?;

        for entity in entities {
            let entity_entry = EntitySerializer::new(self.world, entity);
            s.serialize_element(&entity_entry).unwrap(); // TODO: We don't want missing entities
        }

        s.end()
    }
}


impl<'a> WorldSerializer<'a> {
    pub fn new(world: &'a World) -> Self {
        Self { world }
    }
}

impl<'a> serde::Serialize for WorldSerializer<'a> {
    fn serialize<S: serde::Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        let mut s = serializer.serialize_struct("World", 2)?;
        let entity_entries = EntityEntrySerializer::new(self.world);

        s.serialize_field("id", uuid::Uuid::new_v4().as_ref())?;
        s.serialize_field("ee", &entity_entries)?;

        s.end()
    }
}


