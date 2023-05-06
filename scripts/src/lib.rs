use scr_types::prelude::*;
use tar_ecs::prelude::*;

#[System(Update)]
fn change_transforms(transforms: QueryMut<Transform>) {
    transforms.for_each(|t| t.pos.x += 0.00001);
}

#[InitSystems]
fn init() -> Systems {
    Systems::new().add(change_transforms)
}
