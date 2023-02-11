use crate::prelude::*;

#[derive(Component)]
struct Position {
    x: f32,
    y: f32,
    z: f32
}

impl Position {
    fn new(x: f32, y: f32, z: f32) -> Self {
        Self { x, y, z }
    }
}


#[derive(Component)]
struct UUID {
    id: u128 
}

impl UUID {
    fn new(id: u128) -> Self {
        Self { id }
    }
}


#[derive(Component)]
struct Color {
    r: f32,
    g: f32,
    b: f32,
    a: f32
}

impl Color {
    fn new(r: f32, g: f32, b: f32, a: f32) -> Self {
        Self { r, g, b, a }
    }
}


#[test]
fn single_entity_single_component() {
    let mut world = World::new();

    let entity = world.entity_create();
    world.entity_set(entity, UUID::new(19700101000000));

    assert!(world.entity_get::<UUID>(entity).unwrap().id == 19700101000000);
}


#[test]
fn single_entity_multiple_components_single() {
    let mut world = World::new();

    let entity = world.entity_create();
    world.entity_set(entity, UUID::new(19700101000000));
    world.entity_set(entity, Position::new(16.0, 16.0, 42.0));
    world.entity_set(entity, Color::new(1.0, 0.0, 1.0, 1.0));

    let (uuid, position, color) = (
        world.entity_get::<UUID>(entity).unwrap(),
        world.entity_get::<Position>(entity).unwrap(),
        world.entity_get::<Color>(entity).unwrap()
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
            Color::new(1.0, 0.0, 1.0, 1.0)
        )
    );

    let (uuid, position, color) = {
        let (uuid, position, color) = world.entity_get::<(UUID, Position, Color)>(entity);
        (uuid.unwrap(), position.unwrap(), color.unwrap())
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

