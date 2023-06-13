pub mod model;

use std::error::Error;

use log::warn;
use model::Model;

pub type SomeResult<T> = Result<T, Box<dyn Error + Send + Sync>>;

/// imports models from a gltf file.
pub fn import_models(file_path: &str) -> SomeResult<Vec<Model>> {
    let path = std::path::Path::new(file_path);

    let name = path
        .file_name()
        .unwrap()
        .to_str()
        .unwrap()
        .split('.')
        .collect::<Vec<&str>>()[0];

    let scenes = easy_gltf::load(file_path)?;
    let mut models = Vec::new();
    for (i, scene) in scenes.into_iter().enumerate() {
        if !scene.cameras.is_empty() || !scene.cameras.is_empty() {
            warn!("camera and light importing are not yet supported");
        }
        for model in scene.models {
            models.push(model::Model::new_from_gltf(model, format!("{name}_{i}",)));
        }
    }

    Ok(models)
}
