use std::{collections::HashMap, sync::Arc};

use tar_res::{
    material::PerFrameData, mesh::StaticMesh, object::Object, texture::Texture, Result, WgpuInfo,
};

use crossbeam_channel::{bounded, Receiver};
use tar_types::{
    camera::get_cam_params,
};
use scr_types::{
    components::{Camera, Rendering, Transform},
};
use uuid::Uuid;
use winit::dpi::PhysicalSize;

use crate::GameObject;

const THREAD_NUM: usize = 2;

pub struct ForwardRenderer {
    pub device: Arc<wgpu::Device>,
    pub queue: Arc<wgpu::Queue>,
    // pub cameras: HashMap<uuid::Uuid, crate::camera::RawCamera>,
    pub objects: HashMap<Uuid, Object>,
    pub active_camera: Option<Uuid>,
    pub depth_texture: tar_res::texture::Texture,
    pub format: wgpu::TextureFormat,
    pub threadpool: threadpool::ThreadPool,
    pub receivers: HashMap<Uuid, Receiver<(Object, HashMap<Uuid, StaticMesh>)>>,
    pub meshes: HashMap<Uuid, StaticMesh>,
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
            // cameras: HashMap::new(),
            objects: HashMap::new(),
            active_camera: None,
            depth_texture,
            format,
            threadpool: threadpool::ThreadPool::new(THREAD_NUM),
            receivers: HashMap::new(),
            meshes: HashMap::new(),
        }
    }

    pub fn resize(
        &mut self,
        new_size: winit::dpi::PhysicalSize<u32>,
        config: &mut wgpu::SurfaceConfiguration,
    ) {
        if new_size.width > 0 && new_size.height > 0 {
            config.width = new_size.width;
            config.height = new_size.height;
            // for (_, camera) in &mut self.cameras {
            //     camera.proj.resize(new_size.width, new_size.height);
            // }
            self.depth_texture =
                Texture::create_depth_texture(&self.device, config, "depth_texture");
        }
    }

    // pub fn select_camera(&mut self, cam: uuid::Uuid) {
    //     self.active_camera = Some(cam);
    // }

    pub fn add_object<'a>(&'a mut self, obj: GameObject<'a>, id: uuid::Uuid) {
        let (tx, rx) = bounded(1);
        match obj {
            GameObject::ModelPath(p, name) => {
                let path: String = p.into();
                let name: String = name.into();
                let w_info = Arc::new(WgpuInfo {
                    device: self.device.clone(),
                    queue: self.queue.clone(),
                    surface_format: self.format,
                });
                self.threadpool.execute(move || {
                    let mut meshes = HashMap::new();
                    let path = tar_res::import_gltf(&path, &name).unwrap();
                    let object = tar_res::load_object(path, w_info, &mut meshes).unwrap();
                    tx.send((object, meshes)).unwrap();
                });
            }

            GameObject::ImportedPath(p) => {
                let w_info = Arc::new(WgpuInfo {
                    device: self.device.clone(),
                    queue: self.queue.clone(),
                    surface_format: self.format,
                });
                let path = p.into();
                self.threadpool.execute(move || {
                    let mut meshes = HashMap::new();
                    let object = tar_res::load_object(path, w_info, &mut meshes).unwrap();
                    tx.send((object, meshes)).unwrap();
                });
            }
        }
        self.receivers.insert(id, rx);
    }

    pub fn check_done(
        &mut self,
        id: uuid::Uuid,
    ) -> std::result::Result<bool, crossbeam_channel::RecvError> {
        if let Some(recv) = self.receivers.get(&id) {
            if recv.is_full() {
                let (object, meshes) = recv.recv()?;
                self.objects.insert(id, object);
                for (id, mesh) in meshes {
                    self.meshes.insert(id, mesh);
                }
                self.receivers.remove(&id);
                return Ok(true);
            }
        }
        Ok(false)
    }

    // pub fn add_camera(&mut self, cam: Camera) -> uuid::Uuid {
    //     let id = uuid::Uuid::new_v4();
    //     let camera = cam.cam;
    //     let projection = cam.proj;
    //     let cam_cont = cam.controller;
    //     let mut camera_uniform = CameraUniform::new();
    //     camera_uniform.update_view_proj(&camera, &projection);

    //     self.cameras.insert(
    //         id,
    //         camera::RawCamera {
    //             cam: camera,
    //             proj: projection,
    //             controller: cam_cont,
    //             uniform: camera_uniform,
    //         },
    //     );

    //     id
    // }

    pub fn render(
        &mut self,
        encoder: &mut wgpu::CommandEncoder,
        view: &wgpu::TextureView,
        // rendered_objects: Vec<uuid::Uuid>,
        objects: Vec<(Transform, Rendering)>,
        camera: (Transform, Camera),
        size: PhysicalSize<u32>,
    ) -> Result<()> {
        let mut render_pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
            label: Some("Render Pass"),
            color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                view,
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

        let cam_params = get_cam_params(camera, size);
        let mut data = PerFrameData::default();
        data.u_ambient_light_color = [1.0, 1.0, 1.0];
        data.u_ambient_light_intensity = 0.2;
        data.u_light_color = [5.0, 5.0, 5.0];
        data.u_light_direction = [0.0, 0.5, 0.5];

        //TODO!: this is horrible but the other way round the borrow checker hates it
        // valve pls fix

        for (id, obj) in &mut self.objects {
            if objects.iter().any(|o| o.1.model_id == *id) {
                obj.update_per_frame(&cam_params, &data, &self.queue, &self.meshes)?;
                obj.draw(&mut render_pass, &self.meshes)?;
            }
        }

        Ok(())
    }
}
