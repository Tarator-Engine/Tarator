pub mod model;

use std::error::Error;

use log::warn;
use model::Model;

pub type SomeResult<T> = Result<T, Box<dyn Error + Send + Sync>>;

/// imports models from a gltf file.
pub fn import_models(file_path: &str) -> SomeResult<Vec<Model>> {
    let scenes = easy_gltf::load(file_path)?;
    let mut models = Vec::new();
    for scene in scenes {
        if !scene.cameras.is_empty() || !scene.cameras.is_empty() {
            warn!("camera and light importing are not yet supported");
        }
        for model in scene.models {
            models.push(model::Model::new_from_gltf(model));
        }
    }

    Ok(models)
}
