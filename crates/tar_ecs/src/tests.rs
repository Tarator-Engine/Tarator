use crate::prelude::*;

#[derive(Component)]
struct Test1(String, u32);

#[derive(Component)]
struct Test2(String, u32);

#[test]
fn single_entity() {
    let mut world = World::new();

    let entity = world.entity_new().unwrap();
    let component = (Test1(format!("Yeah!"), 32), Test2(format!("Baby!"), 64));
    world.entity_set(entity, component).unwrap();

    let test1 = world.entity_get::<Test1>(entity).unwrap().lock().unwrap();
    let test2 = world.entity_get::<Test2>(entity).unwrap().lock().unwrap();
    assert!(test1.0 == format!("Yeah!"));
    assert!(test1.1 == 32);
    assert!(test2.0 == format!("Baby!!"));
    assert!(test2.1 == 64);
}

#[test]
fn view() {
    let mut world = World::new();

    for _ in 0..6 {
        let entity = world.entity_new().unwrap();
        let component = (Test1(format!("Yeah!"), 32), Test2(format!("Baby!"), 64));
        world.entity_set(entity, component).unwrap();
    }

    for entity in world.entity_view::<(Test2, Test1)>().unwrap() {
        let test1 = world.entity_get::<Test1>(entity).unwrap().lock().unwrap();
        let test2 = world.entity_get::<Test2>(entity).unwrap().lock().unwrap();
        assert!(test1.0 == format!("Yeah!"));
        assert!(test1.1 == 32);
        assert!(test2.0 == format!("Baby!!"));
        assert!(test2.1 == 64);
    }
}

#[test]
fn query() {
    let mut world = World::new();

    for _ in 0..6 {
        let entity = world.entity_new().unwrap();
        let component = (Test1(format!("Yeah!"), 32), Test2(format!("Baby!"), 64));
        world.entity_set(entity, component).unwrap();
    }

    for (test2, test1) in world.component_view::<(Test2, Test1)>().unwrap() {
        assert!(test1.0 == format!("Yeah!"));
        assert!(test1.1 == 32);
        assert!(test2.0 == format!("Baby!!"));
        assert!(test2.1 == 64);
    }
}

