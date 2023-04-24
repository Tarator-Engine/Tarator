use winit::window::Window;

use tar_shader::shader;

use crate::model::Model;

pub struct RenderState {
    pub surface: wgpu::Surface,
    pub device: wgpu::Device,
    pub queue: wgpu::Queue,
    pub config: wgpu::SurfaceConfiguration,
    pub size: winit::dpi::PhysicalSize<u32>,
    pub window: Window,
    pub global_frame_bind_group: shader::bind_groups::BindGroup0,
    pub models: Vec<Model>,
}
