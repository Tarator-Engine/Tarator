use std::collections::HashMap;

use tar_shader::shader;

use crate::{
    camera,
    model::{texture::DepthTexture, Model},
};

pub struct RenderState {
    // general wgpu data
    pub surface: wgpu::Surface,
    pub device: wgpu::Device,
    pub queue: wgpu::Queue,
    pub adapter: wgpu::Adapter,
    pub config: wgpu::SurfaceConfiguration,

    // data specific to rendering
    pub global_frame_bind_group: shader::bind_groups::BindGroup0,
    pub depth_tex: DepthTexture,
    pub uniform_buffer: wgpu::Buffer,
    pub uniform_data: shader::UniformData,

    // the models for rendering
    pub models: HashMap<uuid::Uuid, Model>,

    // editor camera data
    pub editor_cam: camera::Camera,
    pub editor_cam_controller: camera::CameraController,
    pub editor_projection: camera::Projection,
    pub mouse_pressed: bool,

    // texture which will be renderd to, this will then be presented in egui
    pub render_target_tex: wgpu::Texture,
    pub render_target_tex_view: wgpu::TextureView,
}
