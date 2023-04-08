use std::error::Error;


type SomeResult<T> = Result<T, Box<dyn Error + Send + Sync>>;

/// Loads models from a file.
/// Assumes each scene is a different model
pub fn load_model(file_path: &str) -> SomeResult<()> {
    let scenes = easy_gltf::load(file_path)?;
    for scene in scenes {
        if !scene.cameras.is_empty() || !scene.cameras.is_empty() {
            println!("camera and light importing are not yet supported");
        }

        
    }

    Ok(())
}