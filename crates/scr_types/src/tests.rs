use crate::{
    Component,
    component::{
        ser::{SerializeCallback, SerWorld},
        de::DeWorldBuilder,
        Info, Transform
    }
};
use serde::{Deserialize, Serialize};
use serde_json::de::Deserializer as JsonDeserializer;
use tar_ecs::prelude::World;

#[derive(Debug, Component, Serialize, Deserialize)]
struct Foo {
    foo1: u32,
    foo2: String,
}


#[derive(Debug, Component, Serialize, Deserialize)]
struct Bar(Vec<u32>);


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
            Info {
                id,
                name: "GigachadEntity".into(),
            },
            Transform::default(),
            Foo {
                foo1: 50,
                foo2: "We are baba!".into(),
            },
            Bar(vec![50, 6665, 13407, 324]),
        );
        world.entity_set(entity, data);

        serde_json::to_string(&SerWorld::new(&world, uuid::Uuid::new_v4())).unwrap()
    };

    let deworld = DeWorldBuilder::new()
        .constructor::<Transform>()
        .constructor::<Foo>()
        .constructor::<Bar>()
        .build(&mut JsonDeserializer::from_str(&serialized))
        .unwrap();

    deworld
        .world
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
