use winit::window::Window;

use tar_shader::shader::{self, Vertex};

pub struct RenderState {
    pub surface: wgpu::Surface,
    pub device: wgpu::Device,
    pub queue: wgpu::Queue,
    pub config: wgpu::SurfaceConfiguration,
    pub size: winit::dpi::PhysicalSize<u32>,
    pub window: Window,
    pub diffuse_bind_group: shader::bind_groups::BindGroup0,
}
