pub mod shader;

pub use shader::Vertex;

impl From<&easy_gltf::model::Vertex> for Vertex {
    fn from(value: &easy_gltf::model::Vertex) -> Self {
        let easy_gltf::model::Vertex {
            position,
            normal,
            tangent,
            tex_coords,
        } = value;
        Self {
            position: position.clone().into(),
            normal: normal.clone().into(),
            tangent: tangent.clone().into(),
            tex_coords: tex_coords.clone().into(),
        }
    }
}
