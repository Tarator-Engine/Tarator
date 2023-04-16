pub mod internal;
use scr_types::{components::Transform, InitSystems, System, Systems};

#[System(Update)]
fn test(entities: Vec<Transform>) {
    println!("hello world from the test script");
    for (i, entity) in entities.iter().enumerate() {
        println!("{i}: {entity:?}");
    }
}

#[InitSystems]
fn init() -> Systems {
    Systems::new().add(test)
}
