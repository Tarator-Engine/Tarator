use scr_types::prelude::*;

#[System(Update)]
fn change_transforms(transforms: QueryMut<Transform>, state: GameState) {
    transforms.for_each(|t| t.pos.x += 1.2 * state.dt.as_secs_f32());
}

#[InitSystems]
fn init() -> Systems {
    Systems::new().add(change_transforms)
}
