fn main() {
    test::_main().unwrap();
}


#[allow(unused)]
mod test {
    use tar_ecs::prelude::*;

    #[derive(Component, Debug)]
    struct Transform {
        pub position: [f32; 3],
        pub rotation: [f32; 3],
        pub scale: [f32; 3],
    }

    #[derive(Component, Debug, Clone, Copy)]
    struct Veloctiy(f32, f32, f32);

    pub fn _main() -> Result<(), String> {
        
        let mut world = World::new();
        
        // init the components on the world
        world.component_set::<Transform>()?;
        world.component_set::<Veloctiy>()?;

        // entity composition 1
        let entity = world.entity_new()?;
        world.entity_set::<Transform>(entity)?;
        let mut velocity = *world.entity_set::<Veloctiy>(entity)?;
        velocity.0 = -0.3;
        velocity.1 = -0.1;
        velocity.2 = 0.1;

        // do some printing
        for _ in 0..3 {
            world.entity_operate::<Transform>(entity, |transform| {
                transform.position = [transform.position[0] + velocity.0, transform.position[1] + velocity.1, transform.position[2] + velocity.2];
                println!("{:#?}", transform);
            })?;
        }

        world.entity_destroy(entity)?;

        Ok(())
    }
}

