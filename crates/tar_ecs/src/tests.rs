use std::any::type_name;

use crate::prelude::*;

#[derive(Component, Default)]
struct Position {
    x: f32,
    y: f32,
    z: f32,
}

impl Position {
    fn new(x: f32, y: f32, z: f32) -> Self {
        Self { x, y, z }
    }
}

#[derive(Component)]
struct Label {
    name: String,
}

impl Label {
    fn new(name: impl Into<String>) -> Self {
        Label { name: name.into() }
    }
}

#[derive(Clone, Component, Default)]
struct UUID {
    id: u128,
}

impl UUID {
    fn new(id: u128) -> Self {
        Self { id }
    }
}

#[derive(Component, Default)]
struct Color {
    r: f32,
    g: f32,
    b: f32,
    a: f32,
}

impl Color {
    fn new(r: f32, g: f32, b: f32, a: f32) -> Self {
        Self { r, g, b, a }
    }
}

#[derive(Component, Eq, PartialEq)]
struct Zst;

#[test]
fn single_entity_single_component() {
    let mut world = World::new();

    let entity = world.entity_create();
    world.entity_set(entity, UUID::new(19700101000000));

    let getter = world.entity_get::<UUID>(entity).unwrap();
    assert!(getter.get().id == 19700101000000);
}

#[test]
fn single_entity_multiple_components_single() {
    let mut world = World::new();

    let entity = world.entity_create();
    world.entity_set(entity, UUID::new(19700101000000));
    world.entity_set(entity, Position::new(16.0, 16.0, 42.0));
    world.entity_set(entity, Color::new(1.0, 0.0, 1.0, 1.0));

    let (uuid, position, color) = (
        world.entity_get::<UUID>(entity).unwrap().get(),
        world.entity_get::<Position>(entity).unwrap().get(),
        world.entity_get::<Color>(entity).unwrap().get(),
    );
    assert!(uuid.id == 19700101000000);
    assert!(position.x == 16.0);
    assert!(position.y == 16.0);
    assert!(position.z == 42.0);
    assert!(color.r == 1.0);
    assert!(color.g == 0.0);
    assert!(color.b == 1.0);
    assert!(color.a == 1.0);
}

#[test]
fn single_entity_multiple_components_multi() {
    let mut world = World::new();

    let entity = world.entity_create();
    world.entity_set(
        entity,
        (
            UUID::new(19700101000000),
            Position::new(16.0, 16.0, 42.0),
            Color::new(1.0, 0.0, 1.0, 1.0),
        ),
    );

    let (uuid, position, color) = world
        .entity_get::<(UUID, Position, Color)>(entity)
        .unwrap()
        .get();
    assert!(uuid.id == 19700101000000);
    assert!(position.x == 16.0);
    assert!(position.y == 16.0);
    assert!(position.z == 42.0);
    assert!(color.r == 1.0);
    assert!(color.g == 0.0);
    assert!(color.b == 1.0);
    assert!(color.a == 1.0);
}

#[test]
fn entity_query() {
    let mut world = World::new();

    for _ in 0..5 {
        let entity = world.entity_create();
        world.entity_set(entity, UUID::new(19700101000000));
    }

    for entity in world.entity_collect::<UUID>() {
        let uuid = world.entity_get::<UUID>(entity).unwrap().get();
        assert!(uuid.id == 19700101000000);
    }
}

#[test]
fn component_query() {
    let mut world = World::new();

    for n in 5..10 {
        let entity = world.entity_create();
        world.entity_set(entity, UUID::new(n));
    }

    for n in 0..5 {
        let entity = world.entity_create();
        world.entity_set(entity, (UUID::new(n), Position::new(16.0, 16.0, 42.0)));
    }

    let mut n = 0;
    for uuid in world.component_query::<UUID>() {
        assert!(uuid.id == n, "{} : {}", uuid.id, n);
        n += 1;
    }

    assert!(n == 10, "Expected 10 iterations of UUID, made {}", n);

    for position in world.component_query::<Position>() {
        assert!(position.x == 16.0);
        assert!(position.y == 16.0);
        assert!(position.z == 42.0);
    }
}

#[test]
fn zst() {
    let mut world = World::new();

    let entity = world.entity_create();
    world.entity_set(entity, Zst);

    for query_entity in world.entity_collect::<Zst>() {
        assert!(entity == query_entity);
    }

    for zst in world.component_query::<Zst>() {
        assert!(*zst == Zst);
    }
}

#[test]
fn component_clone() {
    let mut world = World::new();

    for _ in 0..10 {
        let entity = world.entity_create();
        world.entity_set(entity, UUID::new(16));
    }

    for mut uuid in world.component_collect::<UUID>() {
        assert!(uuid.id == 16);
        uuid.id = 42;
    }

    for uuid in world.component_query::<UUID>() {
        assert!(uuid.id == 16);
    }
}

#[test]
fn collect_entity_by_empty_unit() {
    let mut world = World::new();
    let entity = world.entity_create();
    world.entity_set(entity, (Zst, UUID::new(42), Position::new(16., 16., 42.)));

    for _ in world.entity_collect::<()>() {
        let position = world.entity_get::<Position>(entity).unwrap().get();
        let uuid = world.entity_get::<UUID>(entity).unwrap().get();
        assert!(uuid.id == 42);
        assert!(position.x == 16.);
        assert!(position.y == 16.);
        assert!(position.z == 42.);
        return;
    }

    panic!("Should've already returned!");
}

#[test]
fn callback() {
    #[derive(Callback)]
    struct MyCallback(u32);

    impl Callback<Position> for MyCallback {
        fn callback(&mut self, _: &mut Position) {
            self.0 += 1;
        }
    }

    impl Callback<UUID> for MyCallback {
        fn callback(&mut self, _: &mut UUID) {
            self.0 += 1;
        }
    }

    impl Callback<Color> for MyCallback {
        fn callback(&mut self, _: &mut Color) {
            self.0 += 1;
        }
    }

    impl Callback<Zst> for MyCallback {
        fn callback(&mut self, _: &mut Zst) {
            self.0 += 1;
        }
    }

    Position::add_callback::<MyCallback>();
    UUID::add_callback::<MyCallback>();
    Color::add_callback::<MyCallback>();
    Zst::add_callback::<MyCallback>();

    let mut world = World::new();

    for _ in 0..4 {
        let entity = world.entity_create();
        let data = (Position::default(), UUID::default(), Color::default(), Zst);
        world.entity_set(entity, data);
    }

    let mut cb = MyCallback(0);
    for entity in world.entity_collect::<()>() {
        world.entity_callback(entity, &mut cb);
    }

    assert!(cb.0 == 16, "{} != 16", cb.0);
}

#[test]
fn single_entity_single_component_raw() {
    use crate::component::ComponentHashId;

    let mut world = World::new();

    let entity = world.entity_create();
    world.entity_set(entity, UUID::new(19700101000000));

    let (table, index) = world.entity_get_table_and_index(entity).unwrap();
    let hash_id = ComponentHashId::new::<UUID>();

    unsafe {
        assert!(
            (*table
                .write()
                .get_unchecked_raw(hash_id, index)
                .unwrap()
                .cast::<UUID>())
            .id == 19700101000000
        );
    }
}

#[test]
fn single_entity_multiple_components_raw() {
    use crate::component::ComponentHashId;

    let mut world = World::new();

    let entity = world.entity_create();
    world.entity_set(entity, UUID::new(19700101000000));
    world.entity_set(entity, Position::new(16.0, 16.0, 42.0));
    world.entity_set(entity, Color::new(1.0, 0.0, 1.0, 1.0));

    let (uuid, position, color) = unsafe {
        let (table, index) = world.entity_get_table_and_index(entity).unwrap();
        let table = table.read();

        (
            &*table
                .get_unchecked_raw(ComponentHashId::new::<UUID>(), index)
                .unwrap()
                .cast::<UUID>(),
            &*table
                .get_unchecked_raw(ComponentHashId::new::<Position>(), index)
                .unwrap()
                .cast::<Position>(),
            &*table
                .get_unchecked_raw(ComponentHashId::new::<Color>(), index)
                .unwrap()
                .cast::<Color>(),
        )
    };

    assert!(uuid.id == 19700101000000);
    assert!(position.x == 16.0);
    assert!(position.y == 16.0);
    assert!(position.z == 42.0);
    assert!(color.r == 1.0);
    assert!(color.g == 0.0);
    assert!(color.b == 1.0);
    assert!(color.a == 1.0);
}

#[test]
fn query_component_tables_raw() {
    use crate::component::ComponentHashId;

    let mut world = World::new();

    for n in 0..5 {
        let entity = world.entity_create();
        let data = (UUID::new(n), Label::new("Baba"));
        world.entity_set(entity, data);
    }

    for table in world.component_query_tables(type_name::<(Label, UUID)>()) {
        let table = table.read();

        for n in 0..table.len() {
            let (label, uuid) = unsafe {
                (
                    &*table
                        .get_unchecked_raw(ComponentHashId::new::<Label>(), n)
                        .unwrap()
                        .cast::<Label>(),
                    &*table
                        .get_unchecked_raw(ComponentHashId::new::<UUID>(), n)
                        .unwrap()
                        .cast::<UUID>(),
                )
            };

            assert!(n as u128 == uuid.id);
            assert!(format!("Baba") == label.name);
        }
    }
}

#[test]
fn single_entity_set_unset_raw() {
    use crate::component::{ComponentHashId, Components};
    use std::{
        alloc::Layout,
        mem::{needs_drop, ManuallyDrop},
    };

    unsafe {
        unsafe fn _drop<T>(data: *mut u8) {
            data.cast::<T>().drop_in_place()
        }

        Components::init_raw(
            ComponentHashId::new::<UUID>(),
            Layout::new::<UUID>(),
            needs_drop::<UUID>().then_some(_drop::<UUID>),
        );

        Components::init_raw(
            ComponentHashId::new::<Label>(),
            Layout::new::<Label>(),
            needs_drop::<Label>().then_some(_drop::<Label>),
        );

        Components::init_raw(
            ComponentHashId::new::<Zst>(),
            Layout::new::<Zst>(),
            needs_drop::<Zst>().then_some(_drop::<Zst>),
        );
    }

    let mut world = World::new();
    let entity = world.entity_create();

    // Setting
    {
        let data: &[(&str, *mut u8)] = &[
            (
                type_name::<UUID>(),
                &mut ManuallyDrop::new(UUID::new(19700101000000)) as *mut _ as *mut u8,
            ),
            (
                type_name::<Label>(),
                &mut ManuallyDrop::new(Label::new("Baba")) as *mut _ as *mut u8,
            ),
            (
                type_name::<Zst>(),
                &mut ManuallyDrop::new(Zst) as *mut _ as *mut u8,
            ),
        ];
        unsafe { world.entity_set_raw(entity, type_name::<(UUID, Label, Zst)>(), data) };
    }

    // Getting
    {
        let (uuid, label) = unsafe {
            let (table, index) = world.entity_get_table_and_index(entity).unwrap();
            let table = table.read();

            (
                &*table
                    .get_unchecked_raw(ComponentHashId::new::<UUID>(), index)
                    .unwrap()
                    .cast::<UUID>(),
                &*table
                    .get_unchecked_raw(ComponentHashId::new::<Label>(), index)
                    .unwrap()
                    .cast::<Label>(),
            )
        };

        assert!(uuid.id == 19700101000000);
        assert!(label.name == format!("Baba"));
    }

    // Unsetting
    /*{
        world.entity_unset_raw(entity, type_name::<(Label, UUID)>());

        unsafe {
            let (table, index) = world.entity_get_table_and_index(entity).unwrap();
            let table = table.read();

            assert!(table
                .get_unchecked_raw(ComponentHashId::new::<UUID>(), index)
                .is_none());
            assert!(table
                .get_unchecked_raw(ComponentHashId::new::<Label>(), index)
                .is_none());
            assert!(table
                .get_unchecked_raw(ComponentHashId::new::<Zst>(), index)
                .is_some());
        }
    } TODO: unsetting seems to be broken */
}
