pub mod internal;
use scr_types::prelude::*;
use tar_ecs::prelude::*;

#[System(Update)]
fn print_transforms(entities: Query<Transform>) {
    let mut a = 0.0;
    entities.for_each(|transform| a += transform.pos.x);
    println!("{a}");
}

#[System(Update)]
fn change_transforms(transforms: QueryMut<Transform>) {
    transforms.for_each(|t| t.pos.x += 400.0);
}

#[InitSystems]
fn init() -> Systems {
    Systems::new().add(change_transforms).add(print_transforms)
}
