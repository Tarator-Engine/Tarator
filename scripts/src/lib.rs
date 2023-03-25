use tar_abi::System;
use tar_types::components::Transform;

#[System(Update)]
fn test(entities: &mut Vec<(&mut Transform)>) {
    for (i, entity) in entities.iter_mut().enumerate() {
        let a = ((i + 1) * 100) as f32;
        entity.pos.x = a;
        entity.pos.y = a;
        entity.pos.z = a;
    }
}
