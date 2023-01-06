use std::{sync::Arc, vec};

use async_trait::async_trait;

use tar_res::{material::PerFrameData, texture::Texture, WgpuInfo};

use crate::{
    camera::{self, CameraUniform},
    GameObject,
};

use super::Renderer;

/// The forward renderer cl
/// ```
/// let fw = ForwardRenderer::new();
/// ```
/// # Whatever
pub struct ForwardRenderer {
    pub surface: wgpu::Surface,
    pub device: Arc<wgpu::Device>,
    pub queue: Arc<wgpu::Queue>,
    pub config: wgpu::SurfaceConfiguration,
    pub size: winit::dpi::PhysicalSize<u32>,
    pub cameras: Vec<crate::camera::RawCamera>,
    pub objects: Vec<tar_res::object::Object>,
    pub active_camera: Option<u32>,
    pub depth_texture: tar_res::texture::Texture,

    // DEVELOPMENT PURPOSES ONLY // TODO!: REMOVE //
    pub mouse_pressed: bool,
}

#[async_trait]
impl Renderer<'_> for ForwardRenderer {
    async fn new(window: &winit::window::Window) -> Self {
        let size = window.inner_size();

        let instance = wgpu::Instance::new(wgpu::Backends::all());
        let surface = unsafe { instance.create_surface(window) };
        let adapter = instance
            .request_adapter(&wgpu::RequestAdapterOptions {
                power_preference: wgpu::PowerPreference::default(),
                compatible_surface: Some(&surface),
                force_fallback_adapter: false,
            })
            .await
            .unwrap();
        let (device, queue) = adapter
            .request_device(
                &wgpu::DeviceDescriptor {
                    label: None,
                    features: wgpu::Features::empty(),
                    limits: if cfg!(target_arch = "wasm32") {
                        wgpu::Limits::downlevel_webgl2_defaults()
                    } else {
                        wgpu::Limits::default()
                    },
                },
                None,
            )
            .await
            .unwrap();

        let config = wgpu::SurfaceConfiguration {
            usage: wgpu::TextureUsages::RENDER_ATTACHMENT,
            format: surface.get_supported_formats(&adapter)[0],
            width: size.width,
            height: size.height,
            present_mode: wgpu::PresentMode::AutoVsync,
            alpha_mode: wgpu::CompositeAlphaMode::Auto,
        };

        surface.configure(&device, &config);

        let depth_texture = Texture::create_depth_texture(&device, &config, "depth_texture");
        Self {
            surface,
            device: Arc::new(device),
            queue: Arc::new(queue),
            cameras: vec![],
            objects: vec![],
            size,
            config,
            active_camera: None,
            depth_texture,
            mouse_pressed: false,
        }
    }

    fn resize(&mut self, new_size: winit::dpi::PhysicalSize<u32>) {
        if new_size.width > 0 && new_size.height > 0 {
            for camera in &mut self.cameras {
                camera.proj.resize(new_size.width, new_size.height);
                self.size = new_size;
                self.config.width = new_size.width;
                self.config.height = new_size.height;
                self.surface.configure(&self.device, &self.config);
                self.depth_texture =
                    Texture::create_depth_texture(&self.device, &self.config, "depth_texture");
            }
        }
    }

    fn select_camera(&mut self, cam: u32) {
        self.active_camera = Some(cam);
    }

    async fn add_object(&mut self, obj: GameObject<'impl0>) -> tar_res::Result<()> {
        match obj {
            GameObject::Camera(cam) => {
                let camera = cam.cam;
                let projection = cam.proj;
                let cam_cont = cam.controller;
                let mut camera_uniform = CameraUniform::new();
                camera_uniform.update_view_proj(&camera, &projection);

                self.cameras.push(camera::RawCamera {
                    cam: camera,
                    proj: projection,
                    controller: cam_cont,
                    uniform: camera_uniform,
                });
            }

            GameObject::ModelPath(p, name) => {
                let path = tar_res::import_gltf(p, name)?;
                let w_info = Arc::new(WgpuInfo {
                    device: self.device.clone(),
                    queue: self.queue.clone(),
                    surface_format: self.config.format,
                });
                let object = tar_res::load_object(path, w_info)?;
                self.objects.push(object);
            }

            GameObject::ImportedPath(p) => {
                let w_info = Arc::new(WgpuInfo {
                    device: self.device.clone(),
                    queue: self.queue.clone(),
                    surface_format: self.config.format,
                });
                let object = tar_res::load_object(p.into(), w_info)?;
                self.objects.push(object);
            }

            GameObject::Object(obj) => {
                self.objects.push(obj);
            }
        }

        Ok(())
    }

    fn render(&mut self) -> Result<(), wgpu::SurfaceError> {
        let output = self.surface.get_current_texture()?;
        let view = output
            .texture
            .create_view(&wgpu::TextureViewDescriptor::default());

        let mut encoder = self
            .device
            .create_command_encoder(&wgpu::CommandEncoderDescriptor {
                label: Some("Command Encoder"),
            });

        {
            let mut render_pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
                label: Some("Render Pass"),
                color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                    view: &view,
                    resolve_target: None,
                    ops: wgpu::Operations {
                        load: wgpu::LoadOp::Clear(wgpu::Color {
                            r: 0.0,
                            g: 0.0,
                            b: 0.0,
                            a: 0.0,
                        }),
                        store: true,
                    },
                })],
                depth_stencil_attachment: Some(wgpu::RenderPassDepthStencilAttachment {
                    view: &self.depth_texture.view,
                    depth_ops: Some(wgpu::Operations {
                        load: wgpu::LoadOp::Clear(1.0),
                        store: true,
                    }),
                    stencil_ops: None,
                }),
            });
            if let Some(cam) = self.active_camera {
                let cam_params = self.cameras[cam as usize].params();
                let mut data = PerFrameData::default();
                data.u_ambient_light_color = [1.0, 1.0, 1.0];
                data.u_ambient_light_intensity = 1.0;
                data.u_light_color = [1.0, 1.0, 1.0];
                data.u_light_direction = [0.0, 0.5, 0.5];
                for o in &mut self.objects {
                    o.update_per_frame(&cam_params, &data, &self.queue);
                    o.draw(&mut render_pass);
                }
            }
        }
        self.queue.submit(std::iter::once(encoder.finish()));
        output.present();
        Ok(())
    }
}
