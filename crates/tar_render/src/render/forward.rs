use std::{sync::Arc, vec};

use tar_res::{material::PerFrameData, texture::Texture, WgpuInfo};

use crate::{
    camera::{self, CameraUniform},
    GameObject,
};

pub struct ForwardRenderer {
    pub device: Arc<wgpu::Device>,
    pub queue: Arc<wgpu::Queue>,
    pub cameras: Vec<crate::camera::RawCamera>,
    pub objects: Vec<tar_res::object::Object>,
    pub active_camera: Option<u32>,
    pub depth_texture: tar_res::texture::Texture,
    pub format: wgpu::TextureFormat,

    // DEVELOPMENT PURPOSES ONLY // TODO!: REMOVE //
    pub mouse_pressed: bool,
}

impl ForwardRenderer {
    pub async fn new(
        device: Arc<wgpu::Device>,
        queue: Arc<wgpu::Queue>,
        config: &wgpu::SurfaceConfiguration,
        format: wgpu::TextureFormat,
    ) -> Self {
        let depth_texture = Texture::create_depth_texture(&device, config, "depth_texture");
        Self {
            device,
            queue,
            cameras: vec![],
            objects: vec![],
            active_camera: None,
            depth_texture,
            mouse_pressed: false,
            format,
        }
    }

    pub fn resize(
        &mut self,
        new_size: winit::dpi::PhysicalSize<u32>,
        config: &wgpu::SurfaceConfiguration,
    ) {
        if new_size.width > 0 && new_size.height > 0 {
            for camera in &mut self.cameras {
                camera.proj.resize(new_size.width, new_size.height);
                self.depth_texture =
                    Texture::create_depth_texture(&self.device, config, "depth_texture");
            }
        }
    }

    pub fn select_camera(&mut self, cam: u32) {
        self.active_camera = Some(cam);
    }

    pub async fn add_object<'a>(&'a mut self, obj: GameObject<'a>) -> tar_res::Result<()> {
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
                    surface_format: self.format,
                });
                let object = tar_res::load_object(path, w_info)?;
                self.objects.push(object);
            }

            GameObject::ImportedPath(p) => {
                let w_info = Arc::new(WgpuInfo {
                    device: self.device.clone(),
                    queue: self.queue.clone(),
                    surface_format: self.format,
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

    pub fn render(
        &mut self,
        encoder: &mut wgpu::CommandEncoder,
        view: &wgpu::TextureView,
    ) -> Result<(), wgpu::SurfaceError> {
        // let output = surface.get_current_texture()?;
        // let view = output
        //     .texture
        //     .create_view(&wgpu::TextureViewDescriptor::default());

        {
            let mut render_pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
                label: Some("Render Pass"),
                color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                    view: view,
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
        Ok(())
    }
}
