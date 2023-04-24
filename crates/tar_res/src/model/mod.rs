use serde::{Deserialize, Serialize};
use tar_shader::shader::Vertex;

use material::Material;

pub mod material;
pub mod serde_helpers;

#[derive(Debug, Serialize, Deserialize)]
pub struct Model {
    pub vertices: Vec<Vertex>,
    pub indices: Option<Vec<u32>>,
    pub material: material::Material,
}

impl Model {
    pub fn new_from_gltf(model: easy_gltf::Model) -> Self {
        let vertices = model
            .vertices()
            .into_iter()
            .map(|v| Vertex::from(v))
            .collect::<Vec<Vertex>>();
        let indices = model
            .indices()
            .map(|vec| vec.clone().into_iter().map(|i| i as u32).collect());

        let material = Material::new_from_gltf(model.material());

        Self {
            vertices,
            indices,
            material,
        }
    }
}
