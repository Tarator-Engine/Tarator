use tar_abi::System;
use tar_types::components::Transform;

#[System(Update)]
fn test(entities: Iter<(Transform)>) {
    println!("{:?}", entities.next().unwrap().pos);
}
