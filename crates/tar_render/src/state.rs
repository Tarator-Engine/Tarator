use winit::window::Window;

use tar_shader::shader;

use crate::{camera, model::Model};

pub struct RenderState {
    pub surface: wgpu::Surface,
    pub device: wgpu::Device,
    pub queue: wgpu::Queue,
    pub config: wgpu::SurfaceConfiguration,
    pub size: winit::dpi::PhysicalSize<u32>,
    pub window: Window,
    pub global_frame_bind_group: shader::bind_groups::BindGroup0,
    pub models: Vec<Model>,
    pub uniform_buffer: wgpu::Buffer,
    pub uniform_data: shader::UniformData,

    pub editor_cam: camera::Camera,
    pub editor_cam_controller: camera::CameraController,
    pub editor_projection: camera::Projection,

    pub mouse_pressed: bool,
    pub dt: std::time::Duration,
}
