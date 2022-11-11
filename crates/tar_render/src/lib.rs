pub mod camera;
pub mod model;
pub mod resources;
pub mod texture;

use camera::CameraUniform;
use cgmath::prelude::*;
use winit::{
    event::*,
    event_loop::{ControlFlow, EventLoop},
    window::Window,
};

use eframe::egui_wgpu;
use eframe::{
    wgpu,
    wgpu::util::DeviceExt,
};

use std::{sync::Arc};

#[cfg(target_arch = "wasm32")]
use wasm_bindgen::prelude::*;

use model::{DrawLight, DrawModel, Vertex};

use crate::model::Light;
use crate::model::Simplified;

type UUID = u32;

/// the GameObject is a enum which is used to pass something to a
/// Renderer's add_object()
pub enum GameObject<'a> {
    Model(model::Model),
    ModelPath(&'a str, Vec<model::Instance>),
    RawModel(model::RawModel),
    Light(model::Light),
    Camera(camera::Camera),
}

/// Idf is used for identification
/// i.e. you can pass either an index or a UUID
pub enum Idf {
    N(u32),
    ID(UUID),
}

pub struct EditorRenderer {
    last_frame: std::time::Instant,
}


impl EditorRenderer {
    pub fn new<'a>(cc: &'a eframe::CreationContext<'a>) -> Option<Self> {
        // Get the WGPU render state from the eframe creation context. This can also be retrieved
        // from `eframe::Frame` when you don't have a `CreationContext` available.
        let wgpu_render_state = cc.wgpu_render_state.as_ref()?;

        let device = &wgpu_render_state.device;
        let queue = &&wgpu_render_state.queue;

        let config = wgpu::SurfaceConfiguration {
            usage: wgpu::TextureUsages::RENDER_ATTACHMENT,
            format: wgpu_render_state.target_format,
            width: 800,
            height: 800,
            present_mode: wgpu::PresentMode::Fifo,
            //alpha_mode: wgpu::CompositeAlphaMode::Auto,
        };

        let texture_bind_group_layout =
        device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
            entries: &[
                wgpu::BindGroupLayoutEntry {
                    binding: 0,
                    visibility: wgpu::ShaderStages::FRAGMENT,
                    ty: wgpu::BindingType::Texture {
                        multisampled: false,
                        sample_type: wgpu::TextureSampleType::Float { filterable: true },
                        view_dimension: wgpu::TextureViewDimension::D2,
                    },
                    count: None,
                },
                wgpu::BindGroupLayoutEntry {
                    binding: 1,
                    visibility: wgpu::ShaderStages::FRAGMENT,
                    ty: wgpu::BindingType::Sampler(wgpu::SamplerBindingType::Filtering),
                    count: None,
                },
                // normal map
                wgpu::BindGroupLayoutEntry {
                    binding: 2,
                    visibility: wgpu::ShaderStages::FRAGMENT,
                    ty: wgpu::BindingType::Texture {
                        sample_type: wgpu::TextureSampleType::Float { filterable: true },
                        view_dimension: wgpu::TextureViewDimension::D2,
                        multisampled: false,
                    },
                    count: None,
                },
                wgpu::BindGroupLayoutEntry {
                    binding: 3,
                    visibility: wgpu::ShaderStages::FRAGMENT,
                    ty: wgpu::BindingType::Sampler(wgpu::SamplerBindingType::Filtering),
                    count: None,
                },
            ],
            label: Some("texture_bind_group_layout"),
        });

        let camera_bind_group_layout =
            device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
                entries: &[wgpu::BindGroupLayoutEntry {
                    binding: 0,
                    visibility: wgpu::ShaderStages::VERTEX | wgpu::ShaderStages::FRAGMENT,
                    ty: wgpu::BindingType::Buffer {
                        ty: wgpu::BufferBindingType::Uniform,
                        has_dynamic_offset: false,
                        min_binding_size: None,
                    },
                    count: None,
                }],
                label: Some("camera_bind_group_layout"),
            });

        let light_bind_group_layout =
            device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
                entries: &[wgpu::BindGroupLayoutEntry {
                    binding: 0,
                    visibility: wgpu::ShaderStages::VERTEX | wgpu::ShaderStages::FRAGMENT,
                    ty: wgpu::BindingType::Buffer {
                        ty: wgpu::BufferBindingType::Uniform,
                        has_dynamic_offset: false,
                        min_binding_size: None,
                    },
                    count: None,
                }],
                label: None,
            });

        let depth_texture =
            texture::RawTexture::create_depth_texture(&device, &config, "depth_texture");

        let render_pipeline_layout =
            device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
                label: Some("Render Pipeline Layout"),
                bind_group_layouts: &[
                    &texture_bind_group_layout,
                    &camera_bind_group_layout,
                    &light_bind_group_layout,
                ],
                push_constant_ranges: &[],
            });

        let render_pipeline = {
            let shader = wgpu::ShaderModuleDescriptor {
                label: Some("Normal Shader"),
                source: wgpu::ShaderSource::Wgsl(include_str!("shader.wgsl").into()),
            };
            create_render_pipeline(
                &device,
                &render_pipeline_layout,
                config.format,
                Some(texture::RawTexture::DEPTH_FORMAT),
                &[model::ModelVertex::desc(), model::RawInstance::desc()],
                shader,
            )
        };

        let light_render_pipeline = {
            let layout = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
                label: Some("Light Pipeline Layout"),
                bind_group_layouts: &[&camera_bind_group_layout, &light_bind_group_layout],
                push_constant_ranges: &[],
            });
            let shader = wgpu::ShaderModuleDescriptor {
                label: Some("Light Shader"),
                source: wgpu::ShaderSource::Wgsl(include_str!("light.wgsl").into()),
            };
            create_render_pipeline(
                &device,
                &layout,
                config.format,
                None,
                &[model::ModelVertex::desc()],
                shader,
            )
        };
        let mut res = EditorRenderResources {
            render_pipeline,
            light_render_pipeline,
            cameras: vec![],
            models: vec![],
            lights: vec![],
            texture_bind_group_layout,
            active_camera: 0,
            depth_texture,
            light_bind_group_layout,
            camera_bind_group_layout,
        };let int_camera =
        camera::IntCamera::new((0.0, 5.0, 10.0), cgmath::Deg(-90.0), cgmath::Deg(-20.0));
    let projection = camera::Projection::new(
        config.width,
        config.height,
        cgmath::Deg(45.0),
        0.1,
        100.0,
    );
    let camera_controller = camera::CameraController::new(4.0, 0.4);

    let camera = camera::Camera {
        cam: int_camera,
        proj: projection,
        controller: camera_controller,
    };
    const NUM_INSTANCES_PER_ROW: u32 = 10;
    const INSTANCE_DISPLACEMENT: cgmath::Vector3<f32> = cgmath::Vector3::new(
        NUM_INSTANCES_PER_ROW as f32 * 0.5,
        0.0,
        NUM_INSTANCES_PER_ROW as f32 * 0.5,
    );

    let instances = (0..NUM_INSTANCES_PER_ROW)
        .flat_map(|z| {
            (0..NUM_INSTANCES_PER_ROW).map(move |x| {
                let position = cgmath::Vector3 {
                    x: (x as f32) * 3.0,
                    y: 0.0,
                    z: (z as f32) * 3.0,
                } - INSTANCE_DISPLACEMENT;

                let rotation =
                    cgmath::Quaternion::from_axis_angle((0.0, 1.0, 0.0).into(), cgmath::Deg(180.0));
                model::Instance { position, rotation }
            })
        })
        .collect::<Vec<_>>();
    let instance_data = instances
        .iter()
        .map(model::Instance::to_raw)
        .collect::<Vec<_>>();
    let instance_buffer = device
        .create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("Instance Buffer"),
            contents: bytemuck::cast_slice(&instance_data),
            usage: wgpu::BufferUsages::VERTEX,
        });

    let obj_model = pollster::block_on(resources::load_model(
        "cube.obj",
        device,
        queue,
        &res.texture_bind_group_layout,
        instance_buffer,
        NUM_INSTANCES_PER_ROW * NUM_INSTANCES_PER_ROW,
    ))
    .unwrap();

    res.add_object(GameObject::RawModel(obj_model), &device);

    res.add_object(GameObject::Camera(camera), &device);
    res.select_camera(Idf::N(0));

    res.add_object(GameObject::Light(Light {
        pos: [2.0, 2.0, 2.0],
        color: [1.0, 1.0, 1.0],
    }), &device);

        wgpu_render_state
            .egui_rpass
            .write()
            .paint_callback_resources
            .insert(res);

        Some(Self{
            last_frame: std::time::Instant::now(),
        })
    }

    pub fn custom_painting(&mut self, ui: &mut egui::Ui) {
        let (rect, response) =
            ui.allocate_exact_size(egui::Vec2::splat(800.0), egui::Sense::drag());


        let dt = self.last_frame.elapsed();
        self.last_frame = std::time::Instant::now();
        // The callback function for WGPU is in two stages: prepare, and paint.
        //
        // The prepare callback is called every frame before paint and is given access to the wgpu
        // Device and Queue, which can be used, for instance, to update buffers and uniforms before
        // rendering.
        //
        // The paint callback is called after prepare and is given access to the render pass, which
        // can be used to issue draw commands.
        let cb = egui_wgpu::CallbackFn::new()
            .prepare(move |device, queue, paint_callback_resources| {
                let resources: &mut EditorRenderResources = paint_callback_resources.get_mut().unwrap();
                resources.prepare(device, queue, dt);
            })
            .paint(move |_info, render_pass, paint_callback_resources| {
                let resources: &EditorRenderResources = paint_callback_resources.get().unwrap();
                resources.paint(render_pass);
            });

        let callback = egui::PaintCallback {
            rect,
            callback: Arc::new(cb),
        };

        ui.painter().add(callback);
    }
}


pub struct EditorRenderResources {
    pub render_pipeline: wgpu::RenderPipeline,
    pub texture_bind_group_layout: wgpu::BindGroupLayout,
    pub cameras: Vec<camera::RawCamera>,
    pub models: Vec<model::RawModel>,
    pub lights: Vec<model::RawLight>,
    pub active_camera: u32,
    pub depth_texture: texture::RawTexture,
    pub light_bind_group_layout: wgpu::BindGroupLayout,
    pub camera_bind_group_layout: wgpu::BindGroupLayout,
    pub light_render_pipeline: wgpu::RenderPipeline,
}

impl EditorRenderResources {
    fn select_camera(&mut self, cam_idf: Idf) {
        match cam_idf {
            Idf::ID(_id) => todo!(
                "self.active_camera = self.cameras.iter().position(|&x| x.id() == id).unwrap();"
            ),
            Idf::N(n) => self.active_camera = n,
        }
    }

    fn add_object(&mut self, obj: GameObject, device: &wgpu::Device) {
        match obj {
            GameObject::Camera(cam) => {
                let camera = cam.cam;
                let projection = cam.proj;
                let cam_cont = cam.controller;
                let mut camera_uniform = CameraUniform::new();
                camera_uniform.update_view_proj(&camera, &projection);

                let camera_buffer =
                    device
                        .create_buffer_init(&wgpu::util::BufferInitDescriptor {
                            label: Some("Camera Buffer"),
                            contents: bytemuck::cast_slice(&[camera_uniform]),
                            usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
                        });

                let camera_bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
                    layout: &self.camera_bind_group_layout,
                    entries: &[wgpu::BindGroupEntry {
                        binding: 0,
                        resource: camera_buffer.as_entire_binding(),
                    }],
                    label: Some("Camera Bind Group"),
                });

                self.cameras.push(camera::RawCamera {
                    cam: camera,
                    proj: projection,
                    controller: cam_cont,
                    uniform: camera_uniform,
                    buffer: camera_buffer,
                    bind_group: camera_bind_group,
                });
            }

            GameObject::RawModel(rm) => {
                self.models.push(rm);
            }

            GameObject::Light(l) => {
                let uniform = model::RawLightUniform {
                    position: l.pos,
                    _padding: 0,
                    color: l.color,
                    _padding2: 0,
                };

                let buffer = device
                    .create_buffer_init(&wgpu::util::BufferInitDescriptor {
                        label: Some("Light VB"),
                        contents: bytemuck::cast_slice(&[uniform]),
                        usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
                    });
                let bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
                    layout: &self.light_bind_group_layout,
                    entries: &[wgpu::BindGroupEntry {
                        binding: 0,
                        resource: buffer.as_entire_binding(),
                    }],
                    label: None,
                });

                self.lights.push(model::RawLight {
                    uniform,
                    buffer,
                    bind_group,
                })
            }

            _ => todo!("implement rest"),
        }
    }

    fn prepare(&mut self, _device: &wgpu::Device, queue: &wgpu::Queue, dt: std::time::Duration) {
        let cam = &mut self.cameras[self.active_camera as usize];
        cam.controller.update_camera(&mut cam.cam, dt);
        cam.uniform.update_view_proj(&cam.cam, &cam.proj);
        queue.write_buffer(
            &self.cameras[self.active_camera as usize].buffer,
            0,
            bytemuck::cast_slice(&[self.cameras[self.active_camera as usize].uniform]),
        );

        if self.lights.len() == 0 {
            log::warn!("Warning: no lights in scene (nothing to render)");
            return;
        }

        let old_position: cgmath::Vector3<_> = self.lights[0].uniform.position.into();
        self.lights[0].uniform.position =
            (cgmath::Quaternion::from_axis_angle((0.0, 1.0, 0.0).into(), cgmath::Deg(1.0))
                * old_position)
                .into();
        queue.write_buffer(
            &self.lights[0].buffer,
            0,
            bytemuck::cast_slice(&[self.lights[0].uniform]),
        );

    }

    fn paint<'rp>(&'rp self, render_pass: &mut wgpu::RenderPass<'rp>) {
        if self.lights.len() == 0 {
            log::warn!("Warning: no lights in scene (nothing to render)");
        } else {
            for m in &self.models {
                render_pass.set_vertex_buffer(1, m.instance_buffer.slice(..));
                for l in &self.lights {
                    render_pass.set_pipeline(&self.light_render_pipeline);
                    render_pass.draw_light_model(
                        &m,
                        &self.cameras[self.active_camera as usize].bind_group,
                        &l.bind_group,
                    );

                    render_pass.set_pipeline(&self.render_pipeline);
                    render_pass.draw_model_instanced(
                        &m,
                        0..m.instance_num,
                        &self.cameras[self.active_camera as usize].bind_group,
                        &l.bind_group,
                    );
                }
            }
        }

    }
}

/*/// The Renderer describes a generic renderer with the functions
/// - render -> renders a scene to a specified texture
/// - add_object -> adds a GameObject to the renderers scene
/// - update -> takes a function which updates the different objects
/// - resize -> resizes the render target to the give size
pub trait Renderer {
    fn render(&mut self) -> Result<(), wgpu::SurfaceError>;
    fn add_object(&mut self, _obj: GameObject);
    fn resize(&mut self, new_size: winit::dpi::PhysicalSize<u32>);
    //fn update(
    //    &mut self,
    //    update_fn: dyn Fn(&Vec<camera::RawCamera>, &Vec<model::RawModel>, &Vec<model::RawLight>),
    //);
    fn select_camera(&mut self, cam_idf: Idf);
}
*/

/// The Renderer used in the final exported project
pub struct NativeRenderer {
    pub surface: wgpu::Surface,
    pub device: wgpu::Device,
    pub queue: wgpu::Queue,
    pub config: wgpu::SurfaceConfiguration,
    pub size: winit::dpi::PhysicalSize<u32>,
    pub render_pipeline: wgpu::RenderPipeline,
    pub texture_bind_group_layout: wgpu::BindGroupLayout,
    pub cameras: Vec<camera::RawCamera>,
    pub models: Vec<model::RawModel>,
    pub lights: Vec<model::RawLight>,
    pub active_camera: u32,
    pub depth_texture: texture::RawTexture,
    pub light_bind_group_layout: wgpu::BindGroupLayout,
    pub camera_bind_group_layout: wgpu::BindGroupLayout,
    pub light_render_pipeline: wgpu::RenderPipeline,

    // TODO: remove this (only for testing purpouses)
    pub mouse_pressed: bool,
}

/// creates a render pipeline
fn create_render_pipeline(
    device: &wgpu::Device,
    layout: &wgpu::PipelineLayout,
    color_format: wgpu::TextureFormat,
    depth_format: Option<wgpu::TextureFormat>,
    vertex_layouts: &[wgpu::VertexBufferLayout],
    shader: wgpu::ShaderModuleDescriptor,
) -> wgpu::RenderPipeline {
    let shader = device.create_shader_module(shader);

    device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
        label: Some(&format!("{:?}", shader)),
        layout: Some(layout),
        vertex: wgpu::VertexState {
            module: &shader,
            entry_point: "vs_main",
            buffers: vertex_layouts,
        },
        fragment: Some(wgpu::FragmentState {
            module: &shader,
            entry_point: "fs_main",
            targets: &[Some(wgpu::ColorTargetState {
                format: color_format,
                blend: Some(wgpu::BlendState {
                    alpha: wgpu::BlendComponent::REPLACE,
                    color: wgpu::BlendComponent::REPLACE,
                }),
                write_mask: wgpu::ColorWrites::ALL,
            })],
        }),
        primitive: wgpu::PrimitiveState {
            topology: wgpu::PrimitiveTopology::TriangleList,
            strip_index_format: None,
            front_face: wgpu::FrontFace::Ccw,
            cull_mode: Some(wgpu::Face::Back),
            // Setting this to anything other than Fill requires Features::NON_FILL_POLYGON_MODE
            polygon_mode: wgpu::PolygonMode::Fill,
            // Requires Features::DEPTH_CLIP_CONTROL
            unclipped_depth: false,
            // Requires Features::CONSERVATIVE_RASTERIZATION
            conservative: false,
        },
        depth_stencil: depth_format.map(|format| wgpu::DepthStencilState {
            format,
            depth_write_enabled: true,
            depth_compare: wgpu::CompareFunction::Less,
            stencil: wgpu::StencilState::default(),
            bias: wgpu::DepthBiasState::default(),
        }),
        multisample: wgpu::MultisampleState {
            count: 1,
            mask: !0,
            alpha_to_coverage_enabled: false,
        },
        // If the pipeline will be used with a multiview render pass, this
        // indicates how many array layers the attachments will have.
        multiview: None,
    })
}

impl NativeRenderer {
    pub async fn new(window: &Window) -> Self {
        let size = window.inner_size();

        // The insatnce is a handle to our GPU
        // BackendBit::PRIMARY => Vulkan + Metal + DX12 + Browser WebGPU
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
                    // WebGl doesn't support all of wgpu's features, so if
                    // we're building for the web we'll have to disable some.
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
            present_mode: wgpu::PresentMode::Fifo,
            //alpha_mode: wgpu::CompositeAlphaMode::Auto,
        };

        surface.configure(&device, &config);

        let texture_bind_group_layout =
            device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
                entries: &[
                    wgpu::BindGroupLayoutEntry {
                        binding: 0,
                        visibility: wgpu::ShaderStages::FRAGMENT,
                        ty: wgpu::BindingType::Texture {
                            multisampled: false,
                            sample_type: wgpu::TextureSampleType::Float { filterable: true },
                            view_dimension: wgpu::TextureViewDimension::D2,
                        },
                        count: None,
                    },
                    wgpu::BindGroupLayoutEntry {
                        binding: 1,
                        visibility: wgpu::ShaderStages::FRAGMENT,
                        ty: wgpu::BindingType::Sampler(wgpu::SamplerBindingType::Filtering),
                        count: None,
                    },
                    // normal map
                    wgpu::BindGroupLayoutEntry {
                        binding: 2,
                        visibility: wgpu::ShaderStages::FRAGMENT,
                        ty: wgpu::BindingType::Texture {
                            sample_type: wgpu::TextureSampleType::Float { filterable: true },
                            view_dimension: wgpu::TextureViewDimension::D2,
                            multisampled: false,
                        },
                        count: None,
                    },
                    wgpu::BindGroupLayoutEntry {
                        binding: 3,
                        visibility: wgpu::ShaderStages::FRAGMENT,
                        ty: wgpu::BindingType::Sampler(wgpu::SamplerBindingType::Filtering),
                        count: None,
                    },
                ],
                label: Some("texture_bind_group_layout"),
            });

        let camera_bind_group_layout =
            device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
                entries: &[wgpu::BindGroupLayoutEntry {
                    binding: 0,
                    visibility: wgpu::ShaderStages::VERTEX | wgpu::ShaderStages::FRAGMENT,
                    ty: wgpu::BindingType::Buffer {
                        ty: wgpu::BufferBindingType::Uniform,
                        has_dynamic_offset: false,
                        min_binding_size: None,
                    },
                    count: None,
                }],
                label: Some("camera_bind_group_layout"),
            });

        let light_bind_group_layout =
            device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
                entries: &[wgpu::BindGroupLayoutEntry {
                    binding: 0,
                    visibility: wgpu::ShaderStages::VERTEX | wgpu::ShaderStages::FRAGMENT,
                    ty: wgpu::BindingType::Buffer {
                        ty: wgpu::BufferBindingType::Uniform,
                        has_dynamic_offset: false,
                        min_binding_size: None,
                    },
                    count: None,
                }],
                label: None,
            });

        let depth_texture =
            texture::RawTexture::create_depth_texture(&device, &config, "depth_texture");

        let render_pipeline_layout =
            device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
                label: Some("Render Pipeline Layout"),
                bind_group_layouts: &[
                    &texture_bind_group_layout,
                    &camera_bind_group_layout,
                    &light_bind_group_layout,
                ],
                push_constant_ranges: &[],
            });

        let render_pipeline = {
            let shader = wgpu::ShaderModuleDescriptor {
                label: Some("Normal Shader"),
                source: wgpu::ShaderSource::Wgsl(include_str!("shader.wgsl").into()),
            };
            create_render_pipeline(
                &device,
                &render_pipeline_layout,
                config.format,
                Some(texture::RawTexture::DEPTH_FORMAT),
                &[model::ModelVertex::desc(), model::RawInstance::desc()],
                shader,
            )
        };

        let light_render_pipeline = {
            let layout = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
                label: Some("Light Pipeline Layout"),
                bind_group_layouts: &[&camera_bind_group_layout, &light_bind_group_layout],
                push_constant_ranges: &[],
            });
            let shader = wgpu::ShaderModuleDescriptor {
                label: Some("Light Shader"),
                source: wgpu::ShaderSource::Wgsl(include_str!("light.wgsl").into()),
            };
            create_render_pipeline(
                &device,
                &layout,
                config.format,
                Some(texture::RawTexture::DEPTH_FORMAT),
                &[model::ModelVertex::desc()],
                shader,
            )
        };

        // let debug_material = {
        //     let diffuse_bytes = include_bytes!("../res/cobble-diffuse.png");
        //     let normal_bytes = include_bytes!("../res/cobble-normal.png");

        //     let diffuse_texture = texture::RawTexture::from_bytes(
        //         &device,
        //         &queue,
        //         diffuse_bytes,
        //         "res/alt-diffuse.png",
        //         true,
        //     )
        //     .unwrap();

        //     let normal_texture = texture::RawTexture::from_bytes(
        //         &device,
        //         &queue,
        //         normal_bytes,
        //         "res/alt-normal.png",
        //         true,
        //     )
        //     .unwrap();

        //     model::RawMaterial::new(
        //         &device,
        //         "alt-material",
        //         diffuse_texture,
        //         normal_texture,
        //         &texture_bind_group_layout,
        //     )
        //};

        Self {
            surface,
            device,
            queue,
            render_pipeline,
            light_render_pipeline,
            cameras: vec![],
            models: vec![],
            lights: vec![],
            size,
            config,
            texture_bind_group_layout,
            active_camera: 0,
            depth_texture,
            light_bind_group_layout,
            mouse_pressed: false,
            camera_bind_group_layout,
        }
    }

    fn resize(&mut self, new_size: winit::dpi::PhysicalSize<u32>) {
        if new_size.width > 0 && new_size.height > 0 {
            self.cameras[self.active_camera as usize]
                .proj
                .resize(new_size.width, new_size.height);
            self.size = new_size;
            self.config.width = new_size.width;
            self.config.height = new_size.height;
            self.surface.configure(&self.device, &self.config);
            self.depth_texture = texture::RawTexture::create_depth_texture(
                &self.device,
                &self.config,
                "depth_texture",
            );
        }
    }

    fn select_camera(&mut self, cam_idf: Idf) {
        match cam_idf {
            Idf::ID(_id) => todo!(
                "self.active_camera = self.cameras.iter().position(|&x| x.id() == id).unwrap();"
            ),
            Idf::N(n) => self.active_camera = n,
        }
    }

    fn add_object(&mut self, obj: GameObject) {
        match obj {
            GameObject::Camera(cam) => {
                let camera = cam.cam;
                let projection = cam.proj;
                let cam_cont = cam.controller;
                let mut camera_uniform = CameraUniform::new();
                camera_uniform.update_view_proj(&camera, &projection);

                let camera_buffer =
                    self.device
                        .create_buffer_init(&wgpu::util::BufferInitDescriptor {
                            label: Some("Camera Buffer"),
                            contents: bytemuck::cast_slice(&[camera_uniform]),
                            usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
                        });

                let camera_bind_group = self.device.create_bind_group(&wgpu::BindGroupDescriptor {
                    layout: &self.camera_bind_group_layout,
                    entries: &[wgpu::BindGroupEntry {
                        binding: 0,
                        resource: camera_buffer.as_entire_binding(),
                    }],
                    label: Some("Camera Bind Group"),
                });

                self.cameras.push(camera::RawCamera {
                    cam: camera,
                    proj: projection,
                    controller: cam_cont,
                    uniform: camera_uniform,
                    buffer: camera_buffer,
                    bind_group: camera_bind_group,
                });
            }

            GameObject::RawModel(rm) => {
                self.models.push(rm);
            }

            GameObject::Light(l) => {
                let uniform = model::RawLightUniform {
                    position: l.pos,
                    _padding: 0,
                    color: l.color,
                    _padding2: 0,
                };

                let buffer = self
                    .device
                    .create_buffer_init(&wgpu::util::BufferInitDescriptor {
                        label: Some("Light VB"),
                        contents: bytemuck::cast_slice(&[uniform]),
                        usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
                    });
                let bind_group = self.device.create_bind_group(&wgpu::BindGroupDescriptor {
                    layout: &self.light_bind_group_layout,
                    entries: &[wgpu::BindGroupEntry {
                        binding: 0,
                        resource: buffer.as_entire_binding(),
                    }],
                    label: None,
                });

                self.lights.push(model::RawLight {
                    uniform,
                    buffer,
                    bind_group,
                })
            }

            GameObject::ModelPath(p, i) => {
                let instance_data = i
                    .iter()
                    .map(model::Instance::to_raw)
                    .collect::<Vec<_>>();
                let instance_buffer = self
                    .device
                    .create_buffer_init(&wgpu::util::BufferInitDescriptor {
                        label: Some("Instance Buffer"),
                        contents: bytemuck::cast_slice(&instance_data),
                        usage: wgpu::BufferUsages::VERTEX,
                    });
            
                let obj_model = pollster::block_on(resources::load_model(
                    p,
                    &self.device,
                    &self.queue,
                    &self.texture_bind_group_layout,
                    instance_buffer,
                    i.len() as u32,
                ))
                .unwrap();

                self.models.push(obj_model);
            }

            _ => todo!("implement rest"),
        }
    }

    fn render(&mut self) -> Result<(), wgpu::SurfaceError> {
        let output = self.surface.get_current_texture()?;
        let view = output
            .texture
            .create_view(&wgpu::TextureViewDescriptor::default());

        let mut encoder = self
            .device
            .create_command_encoder(&wgpu::CommandEncoderDescriptor {
                label: Some("Render Pass"),
            });

        {
            let mut render_pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
                label: Some("Render Pass"),
                color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                    view: &view,
                    resolve_target: None,
                    ops: wgpu::Operations {
                        load: wgpu::LoadOp::Clear(wgpu::Color {
                            r: 0.1,
                            g: 0.5,
                            b: 0.4,
                            a: 1.0,
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

            if self.lights.len() == 0 {
                log::warn!("Warning: no lights in scene (nothing to render)");
            } else {
                for m in &self.models {
                    render_pass.set_vertex_buffer(1, m.instance_buffer.slice(..));
                    for l in &self.lights {
                        render_pass.set_pipeline(&self.light_render_pipeline);
                        render_pass.draw_light_model(
                            &m,
                            &self.cameras[self.active_camera as usize].bind_group,
                            &l.bind_group,
                        );

                        render_pass.set_pipeline(&self.render_pipeline);
                        render_pass.draw_model_instanced(
                            &m,
                            0..m.instance_num,
                            &self.cameras[self.active_camera as usize].bind_group,
                            &l.bind_group,
                        );
                    }
                }
            }
        }

        self.queue.submit(std::iter::once(encoder.finish()));
        output.present();

        Ok(())
    }
}

fn input(renderer: &mut NativeRenderer, event: &WindowEvent) -> bool {
    match event {
        WindowEvent::KeyboardInput {
            input:
                KeyboardInput {
                    virtual_keycode: Some(key),
                    state,
                    ..
                },
            ..
        } =>  {
            let ret = renderer.cameras[renderer.active_camera as usize]
            .controller
            .process_keyboard(*key, *state);

            if *key == VirtualKeyCode::P {
                renderer.add_object(GameObject::ModelPath("cube.obj", vec![model::Instance {position: cgmath::Vector3 { x: renderer.models.len() as f32 * 2.0, y: 0.0, z: 0.0 }, rotation: cgmath::Quaternion::new(0.0, 0.0, 0.0, 0.0)}]));
            }
            ret
        },
        WindowEvent::MouseWheel { delta, .. } => {
            renderer.cameras[renderer.active_camera as usize]
                .controller
                .process_scroll(delta);
            true
        }
        WindowEvent::MouseInput {
            button: MouseButton::Left,
            state,
            ..
        } => {
            renderer.mouse_pressed = *state == ElementState::Pressed;
            true
        }
        _ => false,
    }
}

fn update(renderer: &mut NativeRenderer, dt: std::time::Duration) {
    let cam = &mut renderer.cameras[renderer.active_camera as usize];
    cam.controller.update_camera(&mut cam.cam, dt);
    cam.uniform.update_view_proj(&cam.cam, &cam.proj);
    renderer.queue.write_buffer(
        &renderer.cameras[renderer.active_camera as usize].buffer,
        0,
        bytemuck::cast_slice(&[renderer.cameras[renderer.active_camera as usize].uniform]),
    );

    if renderer.lights.len() == 0 {
        log::warn!("Warning: no lights in scene (nothing to render)");
        return;
    }

    let old_position: cgmath::Vector3<_> = renderer.lights[0].uniform.position.into();
    renderer.lights[0].uniform.position =
        (cgmath::Quaternion::from_axis_angle((0.0, 1.0, 0.0).into(), cgmath::Deg(1.0))
            * old_position)
            .into();
    renderer.queue.write_buffer(
        &renderer.lights[0].buffer,
        0,
        bytemuck::cast_slice(&[renderer.lights[0].uniform]),
    );
}


pub async fn run() {
    cfg_if::cfg_if! {
        if #[cfg(target_arch = "wasm32")] {
            std::panic::set_hook(Box::new(console_error_panic_hook::hook));
            console_log::init_with_level(log::Level::Info).expect("Couldn't initialize logger");
        }
        else {
            env_logger::init();
        }
    }

    let event_loop = EventLoop::new();
    let title = env!("CARGO_PKG_NAME");
    let window = winit::window::WindowBuilder::new()
        .with_title(title)
        .build(&event_loop)
        .unwrap();

    #[cfg(target_arch = "wasm32")]
    {
        // Winit prevents sizing with CSS, wo we have to set
        // the size manually when on web.
        use winit::dpi::PhysicalSize;
        window.set_inner_size(PhysicalSize::new(450, 400));

        use winit::platform::web::WindowExtWebSys;
        web_sys::window()
            .and_then(|win| win.document())
            .and_then(|doc| {
                let dst = cod.get_element_by_id("wasm-example")?;
                let canvas = web_sys::Element::from(window.canvas());
                dst.append_child(&canvas).ok()?;
                Some(())
            })
            .expect("Couldn't append canvas to document body.");
    }

    let mut renderer = NativeRenderer::new(&window).await;

    let int_camera =
        camera::IntCamera::new((0.0, 5.0, 10.0), cgmath::Deg(-90.0), cgmath::Deg(-20.0));
    let projection = camera::Projection::new(
        renderer.config.width,
        renderer.config.height,
        cgmath::Deg(45.0),
        0.1,
        100.0,
    );
    let camera_controller = camera::CameraController::new(4.0, 0.4);

    let camera = camera::Camera {
        cam: int_camera,
        proj: projection,
        controller: camera_controller,
    };
    const NUM_INSTANCES_PER_ROW: u32 = 10;
    const INSTANCE_DISPLACEMENT: cgmath::Vector3<f32> = cgmath::Vector3::new(
        NUM_INSTANCES_PER_ROW as f32 * 0.5,
        0.0,
        NUM_INSTANCES_PER_ROW as f32 * 0.5,
    );

    let instances = (0..NUM_INSTANCES_PER_ROW)
        .flat_map(|z| {
            (0..NUM_INSTANCES_PER_ROW).map(move |x| {
                let position = cgmath::Vector3 {
                    x: (x as f32) * 3.0,
                    y: 0.0,
                    z: (z as f32) * 3.0,
                } - INSTANCE_DISPLACEMENT;

                let rotation =
                    cgmath::Quaternion::from_axis_angle((0.0, 1.0, 0.0).into(), cgmath::Deg(180.0));
                model::Instance { position, rotation }
            })
        })
        .collect::<Vec<_>>();

    renderer.add_object(GameObject::ModelPath("C:/Users/slackers/rust/Tarator/crates/tar_render/res/cube.obj", instances));

    renderer.add_object(GameObject::Camera(camera));
    renderer.select_camera(Idf::N(0));

    renderer.add_object(GameObject::Light(Light {
        pos: [2.0, 2.0, 2.0],
        color: [1.0, 1.0, 1.0],
    }));

    let mut last_render_time = instant::Instant::now();
    event_loop.run(move |event, _, control_flow| {
        *control_flow = ControlFlow::Poll;
        match event {
            Event::MainEventsCleared => window.request_redraw(),
            // NEW!
            Event::DeviceEvent {
                event: DeviceEvent::MouseMotion{ delta, },
                .. // We're not using device_id currently
            } => if renderer.mouse_pressed {
                renderer.cameras[renderer.active_camera as usize].controller.process_mouse(delta.0, delta.1)
            }
            // UPDATED!
            Event::WindowEvent {
                ref event,
                window_id,
            } if window_id == window.id() && !input(&mut renderer, event) => {
                match event {
                    #[cfg(not(target_arch="wasm32"))]
                    WindowEvent::CloseRequested
                    | WindowEvent::KeyboardInput {
                        input:
                            KeyboardInput {
                                state: ElementState::Pressed,
                                virtual_keycode: Some(VirtualKeyCode::Escape),
                                ..
                            },
                        ..
                    } => *control_flow = ControlFlow::Exit,
                    WindowEvent::Resized(physical_size) => {
                        renderer.resize(*physical_size);
                    }
                    WindowEvent::ScaleFactorChanged { new_inner_size, .. } => {
                        renderer.resize(**new_inner_size);
                    }
                    _ => {}
                }
            }
            // UPDATED!
            Event::RedrawRequested(window_id) if window_id == window.id() => {
                let now = instant::Instant::now();
                let dt = now - last_render_time;
                last_render_time = now;
                update(&mut renderer, dt);
                match renderer.render() {
                    Ok(_) => {}
                    // Reconfigure the surface if it's lost or outdated
                    Err(wgpu::SurfaceError::Lost | wgpu::SurfaceError::Outdated) => renderer.resize(renderer.size),
                    // The system is out of memory, we should probably quit
                    Err(wgpu::SurfaceError::OutOfMemory) => *control_flow = ControlFlow::Exit,
                    // We're ignoring timeouts
                    Err(wgpu::SurfaceError::Timeout) => log::warn!("Surface timeout"),
                }
            }
            _ => ()
        }
    });
}
