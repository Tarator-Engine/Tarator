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

    #[derive(Component, Debug)]
    struct Veloctiy(f32, f32, f32);

    pub fn _main() -> Result<(), String> {
        
        let mut world = World::new();
        
        world.component_set::<Transform>()?;
        world.component_set::<Veloctiy>()?;

        for _ in 0..16 {
            let entity = world.entity_new()?;
            world.entity_set::<(Transform, Veloctiy)>(entity)?;
        }

        Ok(())
    }
}

