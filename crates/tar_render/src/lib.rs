pub mod camera;
// pub mod render;

use std::sync::Arc;

use camera::CameraUniform;
use tar_res::{
    material::PerFrameData, object::Object, texture::Texture, CameraParams, Mat4, Vec3, WgpuInfo,
};
use winit::{
    event::*,
    event_loop::{ControlFlow, EventLoop},
    window::Window,
};

#[cfg(target_arch = "wasm32")]
use wasm_bindgen::prelude::*;

type UUID = u32;

/// the GameObject is a enum which is used to pass something to a
/// Renderer's add_object()
pub enum GameObject<'a> {
    Object(tar_res::object::Object),
    ModelPath(&'a str, &'a str),
    ImportedPath(&'a str),
    Camera(camera::Camera),
}

/// Idf is used for identification
/// i.e. you can pass either an index or a UUID
pub enum Idf {
    N(u32),
    ID(UUID),
}

/// The Renderer used in the final exported project
pub struct NativeRenderer {
    pub surface: wgpu::Surface,
    pub device: Arc<wgpu::Device>,
    pub queue: Arc<wgpu::Queue>,
    pub config: wgpu::SurfaceConfiguration,
    pub size: winit::dpi::PhysicalSize<u32>,
    pub cameras: Vec<camera::RawCamera>,
    pub objects: Vec<Object>,
    pub active_camera: u32,
    pub depth_texture: Texture,

    // TODO: remove this (only for testing purpouses)
    pub mouse_pressed: bool,
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
            present_mode: wgpu::PresentMode::AutoNoVsync,
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
            active_camera: 0,
            depth_texture,
            mouse_pressed: false,
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
            self.depth_texture =
                Texture::create_depth_texture(&self.device, &self.config, "depth_texture");
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

    async fn add_object(&mut self, obj: GameObject<'static>) -> tar_res::Result<()> {
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
            let a: f32 = self.cameras[self.active_camera as usize].proj.aspect;
            let y: f32 = self.cameras[self.active_camera as usize].proj.fovy.0;
            let n: f32 = self.cameras[self.active_camera as usize].proj.znear;
            let pos = self.cameras[self.active_camera as usize].cam.position;
            let cam_params = CameraParams {
                position: Vec3 {
                    x: pos.x,
                    y: pos.y,
                    z: pos.z,
                },
                view_matrix: self.cameras[self.active_camera as usize].cam.calc_matrix(),
                projection_matrix: Mat4::new(
                    1.0 / (a * (0.5 * y).tan()),
                    0.0,
                    0.0,
                    0.0, // NOTE: first column!
                    0.0,
                    1.0 / (0.5 * y).tan(),
                    0.0,
                    0.0,
                    0.0,
                    0.0,
                    -1.0,
                    -1.0,
                    0.0,
                    0.0,
                    -2.0 * n,
                    0.0,
                ),
            };

            let mut data = PerFrameData::default();
            data.u_ambient_light_color = [1.0, 1.0, 1.0];
            data.u_ambient_light_intensity = 1.0;
            data.u_light_color = [1.0, 1.0, 1.0];
            data.u_light_direction = [0.0, 0.5, 0.5];
            for o in &mut self.objects {
                o.update_per_frame(
                    &cam_params,
                    data.u_light_direction,
                    data.u_light_color,
                    data.u_ambient_light_color,
                    data.u_ambient_light_intensity,
                    data.u_alpha_blend,
                    data.u_alpha_cutoff,
                    &self.queue,
                );
                o.draw(&mut render_pass);
            }
        }
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
            let a: f32 = self.cameras[self.active_camera as usize].proj.aspect;
            let y: f32 = self.cameras[self.active_camera as usize].proj.fovy.0;
            let n: f32 = self.cameras[self.active_camera as usize].proj.znear;
            let pos = self.cameras[self.active_camera as usize].cam.position;
            let cam_params = CameraParams {
                position: Vec3 {
                    x: pos.x,
                    y: pos.y,
                    z: pos.z,
                },
                view_matrix: self.cameras[self.active_camera as usize].cam.calc_matrix(),
                projection_matrix: Mat4::new(
                    1.0 / (a * (0.5 * y).tan()),
                    0.0,
                    0.0,
                    0.0, // NOTE: first column!
                    0.0,
                    1.0 / (0.5 * y).tan(),
                    0.0,
                    0.0,
                    0.0,
                    0.0,
                    -1.0,
                    -1.0,
                    0.0,
                    0.0,
                    -2.0 * n,
                    0.0,
                ),
            };

            let mut data = PerFrameData::default();
            data.u_ambient_light_color = [1.0, 1.0, 1.0];
            data.u_ambient_light_intensity = 1.0;
            data.u_light_color = [1.0, 1.0, 1.0];
            data.u_light_direction = [0.0, 0.5, 0.5];
            for o in &mut self.objects {
                o.update_per_frame(
                    &cam_params,
                    data.u_light_direction,
                    data.u_light_color,
                    data.u_ambient_light_color,
                    data.u_ambient_light_intensity,
                    data.u_alpha_blend,
                    data.u_alpha_cutoff,
                    &self.queue,
                );
                o.draw(&mut render_pass);
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
        } => renderer.cameras[renderer.active_camera as usize]
            .controller
            .process_keyboard(*key, *state),
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

    // let instances = (0..NUM_INSTANCES_PER_ROW)
    //     .flat_map(|z| {
    //         (0..NUM_INSTANCES_PER_ROW).map(move |x| {
    //             let position = cgmath::Vector3 {
    //                 x: (x as f32) * 3.0,
    //                 y: 0.0,
    //                 z: (z as f32) * 3.0,
    //             } - INSTANCE_DISPLACEMENT;

    //             let rotation =
    //                 cgmath::Quaternion::from_axis_angle((0.0, 1.0, 0.0).into(), cgmath::Deg(180.0));
    //             model::Instance { position, rotation }
    //         })
    //     })
    //     .collect::<Vec<_>>();

    renderer
        .add_object(GameObject::ImportedPath("assets/helmet.rmp"))
        .await
        .unwrap();

    renderer
        .add_object(GameObject::Camera(camera))
        .await
        .unwrap();
    renderer.select_camera(Idf::N(0));

    let mut last_render_time = instant::Instant::now();
    event_loop.run(move |event, _, control_flow| {
        *control_flow = ControlFlow::Poll;
        match event {
            Event::MainEventsCleared => window.request_redraw(),
            Event::DeviceEvent {
                event: DeviceEvent::MouseMotion{ delta, },
                .. // We're not using device_id currently
            } => if renderer.mouse_pressed {
                renderer.cameras[renderer.active_camera as usize].controller.process_mouse(delta.0, delta.1)
            }
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
