//! This crate will house the #[no_mangle] functions that will be called from the rest of the
//! engine.
//! It will also be the sole accessor of the tar_ecs crate (except for the scripts of course)

use scr_types::{components::Transform, Systems};

#[no_mangle]
pub fn run_systems(systems: &Systems) {
    let mut world = tar_ecs::prelude::World::new();

    let ent = world.entity_create();
    world.entity_set(ent, Transform::default());

    for sys in &systems.systems {
        let system = sys.0;
        system(&mut world);
    }
}
