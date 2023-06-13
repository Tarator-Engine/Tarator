use tar_ecs::prelude::*;
use fxhash::FxBuildHasher;
use erased_serde::Deserializer as EDeserializer;
use std::collections::HashMap;

use super::{Info, SerdeComponent};


trait DeComponentTrait: SerdeComponent {
    fn construct(
        deserializer: &mut dyn EDeserializer,
        world: &mut World,
        entity: Entity,
    ) -> Result<(), erased_serde::Error> {
        let this: Self = erased_serde::deserialize(deserializer)?; // TODO: Use unwrap_or_default instad?
        world.entity_set(entity, this);

        Ok(())
    }
}

impl<T: SerdeComponent> DeComponentTrait for T {}


type ConstructorFunc = fn(&mut dyn EDeserializer, &mut World, Entity) -> Result<(), erased_serde::Error>;
type ConstructorMap<'a> = HashMap<&'a str, ConstructorFunc, FxBuildHasher>;

pub struct DeWorld {
    pub id: uuid::Uuid,
    pub world: World,
}

#[derive(Default)]
pub struct DeWorldBuilder<'a> {
    constuctors: ConstructorMap<'a>,
}

impl<'a> DeWorldBuilder<'a> {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn constructor<T: SerdeComponent>(mut self) -> Self {
        self.constuctors.insert(T::NAME, T::construct);
        self
    }

    pub fn build<'de, D: serde::Deserializer<'de>>(
        self,
        deserializer: D,
    ) -> Result<DeWorld, D::Error> {
        use serde::de::DeserializeSeed;

        self.deserialize(deserializer)
    }
}

impl<'a, 'de> serde::de::DeserializeSeed<'de> for DeWorldBuilder<'a> {
    type Value = DeWorld;

    fn deserialize<D: serde::Deserializer<'de>>(
        self,
        deserializer: D,
    ) -> Result<Self::Value, D::Error> {
        deserializer.deserialize_struct(
            "World",
            &["id", "ee"],
            DeWorldVisitor {
                constuctors: self.constuctors,
            },
        )
    }
}

struct DeWorldVisitor<'a> {
    constuctors: ConstructorMap<'a>,
}

struct DeEntityEntry<'a> {
    constuctors: &'a ConstructorMap<'a>,
}

struct DeEntityEntryVisitor<'a> {
    constuctors: &'a ConstructorMap<'a>,
}

struct DeEntity<'a> {
    world: &'a mut World,
    constuctors: &'a ConstructorMap<'a>,
}

struct DeEntityVisitor<'a> {
    world: &'a mut World,
    constuctors: &'a ConstructorMap<'a>,
}

struct DeComponents<'a> {
    entity: Entity,
    world: &'a mut World,
    constuctors: &'a ConstructorMap<'a>,
}

struct DeComponentsVisitor<'a> {
    entity: Entity,
    world: &'a mut World,
    constuctors: &'a ConstructorMap<'a>,
}

struct DeComponent<'a> {
    entity: Entity,
    world: &'a mut World,
    func: ConstructorFunc,
}

impl<'a, 'de> serde::de::DeserializeSeed<'de> for DeComponent<'a> {
    type Value = ();

    fn deserialize<D: serde::Deserializer<'de>>(
        self,
        deserializer: D,
    ) -> Result<Self::Value, D::Error> {
        let d = &mut <dyn erased_serde::Deserializer>::erase(deserializer);
        (self.func)(d, self.world, self.entity)
            .map_err(|e| serde::de::Error::custom(format!("Could not parse component: {e}")))
    }
}

impl<'a, 'de> serde::de::Visitor<'de> for DeComponentsVisitor<'a> {
    type Value = ();

    fn expecting(&self, formatter: &mut std::fmt::Formatter) -> std::fmt::Result {
        formatter.write_str("Could not parse components")
    }

    fn visit_map<A: serde::de::MapAccess<'de>>(self, mut map: A) -> Result<Self::Value, A::Error> {
        while let Some(key) = map.next_key::<&str>()? {
            map.next_value_seed(DeComponent {
                entity: self.entity,
                world: self.world,
                func: *self
                    .constuctors
                    .get(key)
                    .ok_or(serde::de::Error::custom(format!(
                        "Component constructor for {key} not initialized"
                    )))?,
            })?
        }

        Ok(())
    }
}

impl<'a, 'de> serde::de::DeserializeSeed<'de> for DeComponents<'a> {
    type Value = ();

    fn deserialize<D: serde::Deserializer<'de>>(
        self,
        deserializer: D,
    ) -> Result<Self::Value, D::Error> {
        deserializer.deserialize_map(DeComponentsVisitor {
            world: self.world,
            entity: self.entity,
            constuctors: self.constuctors,
        })
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
                "id" => {
                    id = Some(uuid::Uuid::parse_str(map.next_value()?).map_err(|e| {
                        serde::de::Error::custom(format!("Could not parse world id: {}", e))
                    })?)
                }
                "name" => name = Some(map.next_value()?),
                "components" => map.next_value_seed(DeComponents {
                    world: self.world,
                    entity,
                    constuctors: self.constuctors,
                })?,
                _ => (),
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

    fn deserialize<D: serde::Deserializer<'de>>(
        self,
        deserializer: D,
    ) -> Result<Self::Value, D::Error> {
        deserializer.deserialize_struct(
            "Entity",
            &["id", "name", "components"],
            DeEntityVisitor {
                world: self.world,
                constuctors: self.constuctors,
            },
        )
    }
}

impl<'a, 'de> serde::de::Visitor<'de> for DeEntityEntryVisitor<'a> {
    type Value = World;

    fn expecting(&self, formatter: &mut std::fmt::Formatter) -> std::fmt::Result {
        formatter.write_str("Could not parse entity entries")
    }

    fn visit_seq<A: serde::de::SeqAccess<'de>>(self, mut seq: A) -> Result<Self::Value, A::Error> {
        let mut world = World::new();

        while let Some(()) = seq.next_element_seed(DeEntity {
            world: &mut world,
            constuctors: &self.constuctors,
        })? {}

        Ok(world)
    }
}

impl<'a, 'de> serde::de::DeserializeSeed<'de> for DeEntityEntry<'a> {
    type Value = World;

    fn deserialize<D: serde::Deserializer<'de>>(
        self,
        deserializer: D,
    ) -> Result<Self::Value, D::Error> {
        deserializer.deserialize_seq(DeEntityEntryVisitor {
            constuctors: self.constuctors,
        })
    }
}

impl<'a, 'de> serde::de::Visitor<'de> for DeWorldVisitor<'a> {
    type Value = DeWorld;

    fn expecting(&self, formatter: &mut std::fmt::Formatter) -> std::fmt::Result {
        formatter.write_str("Could not parse world")
    }

    fn visit_seq<A: serde::de::SeqAccess<'de>>(self, mut seq: A) -> Result<Self::Value, A::Error> {
        let id = uuid::Uuid::parse_str(
            seq.next_element()?
                .ok_or(serde::de::Error::invalid_length(0, &self))?,
        )
        .map_err(|e| serde::de::Error::custom(format!("Could not parse world id: {}", e)))?;
        let world = seq
            .next_element_seed(DeEntityEntry {
                constuctors: &self.constuctors,
            })?
            .ok_or(serde::de::Error::invalid_length(1, &self))?;

        Ok(DeWorld { id, world })
    }

    fn visit_map<A: serde::de::MapAccess<'de>>(self, mut map: A) -> Result<Self::Value, A::Error> {
        let mut id = None;
        let mut world = None;

        while let Some(key) = map.next_key()? {
            match key {
                "id" => {
                    id = Some(uuid::Uuid::parse_str(map.next_value()?).map_err(|e| {
                        serde::de::Error::custom(format!("Could not parse world id: {}", e))
                    })?)
                }
                "ee" => {
                    world = Some(map.next_value_seed(DeEntityEntry {
                        constuctors: &self.constuctors,
                    })?)
                }
                _ => (),
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
