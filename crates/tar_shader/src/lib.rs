use shader::Vertex;

pub mod shader;

impl From<easy_gltf::model::Vertex> for Vertex {
    fn from(value: easy_gltf::model::Vertex) -> Self {
        let easy_gltf::model::Vertex {
            position,
            normal,
            tangent,
            tex_coords,
        } = value;
        Self {
            position: position.into(),
            normal: normal.into(),
            tangent: tangent.into(),
            tex_coords: tex_coords.into(),
        }
    }
}

impl Vertex {
    pub fn desc<'a>() -> wgpu::VertexBufferLayout<'a> {
        wgpu::VertexBufferLayout {
            array_stride: std::mem::size_of::<Vertex>() as wgpu::BufferAddress,
            step_mode: wgpu::VertexStepMode::Vertex,
            attributes: &Self::VERTEX_ATTRIBUTES,
        }
    }
}
