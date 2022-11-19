fn main() {
    test::_main().unwrap();
}


#[allow(unused)]
mod test {
    use tar_ecs::prelude::*;

    #[derive(Component, Default, Debug)]
    struct Transform {
        pub position: [f32; 3],
        pub rotation: [f32; 3],
        pub scale: [f32; 3],
    }

    #[derive(Component, Default, Debug)]
    struct Color {
        pub color: [u32; 4]
    }

    #[derive(Component, Default, Debug)]
    struct Mesh {
        pub mesh: &'static [f32],
    }


    pub fn _main() -> Result<(), &'static str> {
        
        let mut world = World::new();
        
        // init the components on the world
        world.component_set::<Transform>();
        world.component_set::<Color>();
        world.component_set::<Mesh>();

        // entity composition 1
        let e1 = world.entity_new();
        world.entity_set::<Transform>(e1)?;
        world.entity_set::<Color>(e1)?;
        world.entity_set::<Mesh>(e1)?;

        // entity composition 2
        let e2 = world.entity_new();
        world.entity_new();
        world.entity_set::<Transform>(e2)?;
        world.entity_set::<Color>(e2)?;


        // do some printing
        for _ in 0..9999 {
            let transform = world.entity_get_mut::<Transform>(e1)?;
            transform.scale = [transform.scale[0] + 0.5, transform.scale[1] + 0.7, transform.scale[2] + 0.9];
            world.entity_set::<Transform>(e2)?;
            world.entity_unset::<Transform>(e2)?;
        }

        Ok(())
    }
}

