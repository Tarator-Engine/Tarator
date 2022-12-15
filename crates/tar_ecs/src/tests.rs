use crate::prelude::*;

#[test]
fn set_components() {
    #[derive(Component)]
    struct Test1(String, u32);
    #[derive(Component)]
    struct Test2(String, u32);
    let mut world = World::new().unwrap();
    let entity = world.entity_new().unwrap();
    let component = (Test1(format!("Yeah!"), 32), Test2(format!("Baby!"), 64));
    world.entity_set(entity, component).unwrap();
}

