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
        Vertex {
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
    pub fn desc<'a>() -> wgpu::VertexBufferLayout<'a> {
        use std::mem;
        wgpu::VertexBufferLayout {
            array_stride: mem::size_of::<Vertex>() as wgpu::BufferAddress,
            step_mode: wgpu::VertexStepMode::Vertex,
            attributes: &[
                // position
                wgpu::VertexAttribute {
                    offset: 0,
                    shader_location: 0,
                    format: wgpu::VertexFormat::Float32x3,
                },
                // normal
                wgpu::VertexAttribute {
                    offset: mem::size_of::<[f32; 3]>() as wgpu::BufferAddress, // offset of position(3)
                    shader_location: 1,
                    format: wgpu::VertexFormat::Float32x3,
                },
                // tangent
                wgpu::VertexAttribute {
                    offset: mem::size_of::<[f32; 6]>() as wgpu::BufferAddress, // offset of position(3) + normal(3)
                    shader_location: 2,
                    format: wgpu::VertexFormat::Float32x4,
                },
                // tex_coords_0
                wgpu::VertexAttribute {
                    offset: mem::size_of::<[f32; 10]>() as wgpu::BufferAddress, // offset of position(3) + normal(3) + tangent(4)
                    shader_location: 3,
                    format: wgpu::VertexFormat::Float32x2,
                },
                // tex_coords_1
                wgpu::VertexAttribute {
                    offset: mem::size_of::<[f32; 12]>() as wgpu::BufferAddress, // offset of position(3) + normal(3) + tangent(4) + tex_coord_0(2)
                    shader_location: 4,
                    format: wgpu::VertexFormat::Float32x2,
                },
                // color_0
                wgpu::VertexAttribute {
                    offset: mem::size_of::<[f32; 14]>() as wgpu::BufferAddress, // offset of position(3) + normal(3) + tangent(4) + tex_coord_0(2) + tex_coord_1(2)
                    shader_location: 5,
                    format: wgpu::VertexFormat::Float32x4,
                },
                // joints_0
                wgpu::VertexAttribute {
                    offset: mem::size_of::<[f32; 18]>() as wgpu::BufferAddress, // offset of position(3) + normal(3) + tangent(4) + tex_coord_0(2) + tex_coord_1(2) + color_0(4)
                    shader_location: 6,
                    format: wgpu::VertexFormat::Float32x4,
                },
                // weights_0
                wgpu::VertexAttribute {
                    offset: mem::size_of::<[f32; 22]>() as wgpu::BufferAddress, // offset of position(3) + normal(3) + tangent(4) + tex_coord_0(2) + tex_coord_1(2) + color_0(4) + joints_0(4)
                    shader_location: 7,
                    format: wgpu::VertexFormat::Float32x4,
                },
            ],
        }
    }
}
