use winit::window::Window;

use tar_shader::shader::{self, Vertex};

pub struct RenderState {
    pub surface: wgpu::Surface,
    pub device: wgpu::Device,
    pub queue: wgpu::Queue,
    pub config: wgpu::SurfaceConfiguration,
    pub size: winit::dpi::PhysicalSize<u32>,
    pub window: Window,
    pub render_pipeline: wgpu::RenderPipeline,
    pub vertex_buffer: wgpu::Buffer,
    pub index_buffer: wgpu::Buffer,
    pub num_vertices: u32,
    pub num_indices: u32,
    pub diffuse_bind_group: shader::bind_groups::BindGroup0,
}

pub const VERTICES: &[Vertex] = &[
    Vertex {
        position: [-0.0868241, 0.49240386, 0.0],
        normal: [0.5, 0.0, 0.5],
        tangent: [0.0; 4],
        tex_coords: [0.0; 2],
    },
    Vertex {
        position: [-0.49513406, 0.06958647, 0.0],
        normal: [0.5, 0.0, 0.5],
        tangent: [0.0; 4],
        tex_coords: [0.0; 2],
    },
    Vertex {
        position: [-0.21918549, -0.44939706, 0.0],
        normal: [0.5, 0.0, 0.5],
        tangent: [0.0; 4],
        tex_coords: [0.0; 2],
    },
    Vertex {
        position: [0.35966998, -0.3473291, 0.0],
        normal: [0.5, 0.0, 0.5],
        tangent: [0.0; 4],
        tex_coords: [0.0; 2],
    },
    Vertex {
        position: [0.44147372, 0.2347359, 0.0],
        normal: [0.5, 0.0, 0.5],
        tangent: [0.0; 4],
        tex_coords: [0.0; 2],
    },
];

pub const INDICES: &[u16] = &[0, 1, 4, 1, 2, 4, 2, 3, 4];
