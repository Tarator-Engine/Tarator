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
        for _ in 0..20 {
            let entity = world.entity_new()?;
            world.entity_set::<Transform>(entity)?;
            let velocity = world.entity_set::<Veloctiy>(entity)?;
            velocity.0 = -0.3;
            velocity.1 = -0.1;
            velocity.2 = 0.1;
        }
        // another entity for checking the ones above
        {
            let entity = world.entity_new()?;
            // uncomment to check the for loop for functionality
            // world.entity_set::<Veloctiy>(entity)?;
            let transform = world.entity_set::<Transform>(entity)?;
            transform.scale = [420.0, 420.0, 420.0];
        }

        for entity in world.view::<(Transform, Veloctiy)>()? {
            world.entity_operate::<Transform>(entity, |transform| {
                let pos = transform.position;
                println!("{:#?}", transform);
                transform.position = [pos[0] + 2.0, pos[1] + 2.0, pos[2] + 2.0];
            })?;
        }

        Ok(())
    }
}

