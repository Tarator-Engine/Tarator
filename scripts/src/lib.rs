pub mod internal;
use scr_types::prelude::*;
use tar_ecs::prelude::*;

#[System(Update)]
fn print_transforms(entities: Query<Transform>) {
    entities.for_each(|transform| println!("{transform:?}"));
}

#[System(Update)]
fn change_transforms(transforms: Query<Transform>) {
    transforms.for_each(|t| t.pos.x += 400.0);
}

#[InitSystems]
fn init() -> Systems {
    Systems::new().add(change_transforms).add(print_transforms)
}
