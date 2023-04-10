use serde::{Deserialize, Serialize};
use tar_shader::shader::Vertex;

use material::Material;

pub mod material;
pub mod serde_helpers;

#[derive(Debug, Serialize, Deserialize)]
pub struct Model {
    pub vertices: Vec<Vertex>,
    pub indices: Option<Vec<usize>>,
    pub material: material::Material,
}

impl Model {
    pub fn new_from_gltf(model: easy_gltf::Model) -> Self {
        let vertices = model
            .vertices()
            .iter()
            .map(|v| Vertex::from(v.clone()))
            .collect::<Vec<Vertex>>();
        let indices = model.indices().map(|vec| vec.clone());

        let material = Material::new_from_gltf(model.material());

        Self {
            vertices,
            indices,
            material,
        }
    }
}
