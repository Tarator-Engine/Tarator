use bytemuck::{Pod, Zeroable};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, Pod, Zeroable, Serialize, Deserialize)]
#[repr(C)]
pub struct Vertex {
    pub position: [f32; 3],
    pub normal: [f32; 3],
    pub tangent: [f32; 4],
    pub tex_coord_0: [f32; 2],
    pub tex_coord_1: [f32; 2],
    pub color_0: [f32; 4],
    pub joints_0: [u32; 4],
    pub weights_0: [f32; 4],
}

impl Default for Vertex {
    fn default() -> Self {
        Self {
            position: [0.0; 3],
            normal: [0.0; 3],
            tangent: [0.0; 4],
            tex_coord_0: [0.0; 2],
            tex_coord_1: [0.0; 2],
            color_0: [0.0; 4],
            joints_0: [0; 4],
            weights_0: [0.0; 4],
        }
    }
}

impl Vertex {
    const ATTRIBS: [wgpu::VertexAttribute; 8] = wgpu::vertex_attr_array![0 => Float32x3, 1 => Float32x3, 2 => Float32x4, 3 => Float32x2, 4 => Float32x2, 5 => Float32x4, 6 => Float32x4, 7 => Float32x4];
    #[must_use] pub fn desc<'a>() -> wgpu::VertexBufferLayout<'a> {
        use std::mem;
        wgpu::VertexBufferLayout {
            array_stride: mem::size_of::<Self>() as wgpu::BufferAddress,
            step_mode: wgpu::VertexStepMode::Vertex,
            attributes: &Self::ATTRIBS,
        }
    }
}
