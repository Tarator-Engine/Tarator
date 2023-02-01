mod error_msg;
mod pre_render;
mod render;
mod state;

use std::sync::{Arc, Barrier};

use egui::ClippedPrimitive;
use egui::FullOutput;
use instant::Duration;
use parking_lot::Mutex;

use parking_lot::RwLock;
use winit::{
    event::*,
    event_loop::{ControlFlow, EventLoop},
};

#[derive(Debug)]
pub struct DoubleBuffer<T: Clone> {
    pub state: T,
}

impl<T: Clone> DoubleBuffer<T> {
    pub fn update_read(&mut self) -> T {
        return self.state.clone();
    }
}

#[derive(Clone)]
pub struct EngineState {
    dt: Duration,
    fps: u32,
    size: winit::dpi::PhysicalSize<u32>,
    halt: bool,
    view_rect: winit::dpi::PhysicalSize<u32>,
    cam_sensitivity: f32,
    paint_jobs: Vec<ClippedPrimitive>,
    egui_textures_delta: egui::epaint::textures::TexturesDelta,
}

fn input(renderer: &mut tar_render::render::forward::ForwardRenderer, event: &WindowEvent) -> bool {
    match event {
        WindowEvent::KeyboardInput {
            input:
                KeyboardInput {
                    virtual_keycode: Some(key),
                    state,
                    ..
                },
            ..
        } => renderer.cameras[renderer.active_camera.unwrap() as usize]
            .controller
            .process_keyboard(*key, *state),
        WindowEvent::MouseWheel { delta, .. } => {
            renderer.cameras[renderer.active_camera.unwrap() as usize]
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

fn update(renderer: &mut tar_render::render::forward::ForwardRenderer, dt: std::time::Duration) {
    let cam = &mut renderer.cameras[renderer.active_camera.unwrap() as usize];
    cam.controller.update_camera(&mut cam.cam, dt);
    cam.uniform.update_view_proj(&cam.cam, &cam.proj);
}

async fn build_renderer_extras(
    window: &winit::window::Window,
) -> (
    tar_render::render::forward::ForwardRenderer,
    Arc<wgpu::Device>,
    Arc<wgpu::Queue>,
    winit::dpi::PhysicalSize<u32>,
    wgpu::Surface,
    wgpu::SurfaceConfiguration,
    wgpu::Adapter,
) {
    let instance = wgpu::Instance::new(wgpu::Backends::PRIMARY);
    let surface = unsafe { instance.create_surface(&window) };

    let adapter = instance
        .request_adapter(&wgpu::RequestAdapterOptions {
            power_preference: wgpu::PowerPreference::HighPerformance,
            compatible_surface: Some(&surface),
            force_fallback_adapter: false,
        })
        .await
        .unwrap();

    let (device, queue) = adapter
        .request_device(
            &wgpu::DeviceDescriptor {
                features: wgpu::Features::default(),
                limits: wgpu::Limits::default(),
                label: None,
            },
            None,
        )
        .await
        .unwrap();

    let device = Arc::new(device);
    let queue = Arc::new(queue);

    let size = window.inner_size();
    let config = wgpu::SurfaceConfiguration {
        usage: wgpu::TextureUsages::RENDER_ATTACHMENT,
        format: surface.get_supported_formats(&adapter)[0],
        width: size.width,
        height: size.height,
        present_mode: wgpu::PresentMode::AutoNoVsync,
        alpha_mode: surface.get_supported_alpha_modes(&adapter)[0],
    };
    surface.configure(&device, &config);

    let game_renderer = tar_render::render::forward::ForwardRenderer::new(
        device.clone(),
        queue.clone(),
        &config,
        config.format,
    )
    .await;

    return (game_renderer, device, queue, size, surface, config, adapter);
}

pub async fn run() {
    let db = DoubleBuffer {
        state: EngineState {
            dt: Duration::from_secs(0),
            fps: 0,
            size: winit::dpi::PhysicalSize::new(0, 0),
            halt: false,
            view_rect: winit::dpi::PhysicalSize::new(0, 0),
            cam_sensitivity: 0.4,
            paint_jobs: vec![],
            egui_textures_delta: egui::epaint::textures::TexturesDelta::default(),
        },
    };
    let db = Arc::new(Mutex::new(db));

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
        .with_fullscreen(Some(winit::window::Fullscreen::Borderless(None)))
        .build(&event_loop)
        .unwrap();

    #[cfg(target_arch = "wasm32")]
    {
        // Winit prevents sizing with CSS, so we have to set
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

    let mut view_rect = (800, 800);

    let blocking_threads = threadpool::ThreadPool::new(4);

    let barrier = Arc::new(Barrier::new(2));

    let stop = Arc::new(RwLock::new(false));

    let (game_renderer, device, queue, size, surface, config, adapter) =
        build_renderer_extras(&window).await;

    let config = Arc::new(config);
    let size = Arc::new(size);

    let surface = Arc::new(surface);

    let egui_renderer =
        egui_wgpu::Renderer::new(&device, surface.get_supported_formats(&adapter)[0], None, 1);

    let mut egui_state = egui_winit::State::new(&event_loop);

    let r_barrier = barrier.clone();
    let render_s = stop.clone();
    let s_clone = surface.clone();
    let d_clone = device.clone();
    let q_clone = queue.clone();
    let engine_state = db.clone();
    let render_thread = std::thread::spawn(move || {
        render::render_fn(
            r_barrier,
            engine_state,
            game_renderer,
            egui_renderer,
            config,
            s_clone,
            d_clone,
            q_clone,
        );
    });
    let mut errors: Vec<Box<dyn std::error::Error>> = vec![];

    let context = egui::Context::default();

    let mut halt = false;

    let mut winit_events = vec![];

    event_loop.run(move |event, _, control_flow| {
        match event {
            Event::RedrawRequested(..) => {
                let state = &mut db.lock().state;

                let input = egui_state.take_egui_input(&window);
                context.begin_frame(input);

                let mut remove = vec![];
                for (i, err) in (&errors).iter().enumerate() {
                    if error_msg::error_message(&context, err) {
                        remove.push(i);
                    };
                }
                for r in remove.iter().rev() {
                    errors.remove(*r);
                }

                egui::Window::new("Timings")
                    .resizable(false)
                    .show(&context, |ui| {
                        ui.label("Here you can see different frame timings");
                        ui.label(format!("Frame time: {:?}", state.dt));
                        ui.label(format!("FPS: {}", state.fps));
                    });
                egui::SidePanel::right("right panel")
                    .resizable(true)
                    .default_width(300.0)
                    .show(&context, |ui| {
                        ui.vertical_centered(|ui| ui.heading("right panel"))
                    });
                egui::SidePanel::left("left panel")
                    .resizable(true)
                    .default_width(300.0)
                    .show(&context, |ui| {
                        ui.vertical_centered(|ui| ui.heading("left panel"));
                        ui.label("sensitvity");
                        ui.add(egui::Slider::new(&mut state.cam_sensitivity, 0.0..=5.0));
                    });
                egui::TopBottomPanel::bottom("bottom panel")
                    .resizable(true)
                    .default_height(200.0)
                    .show(&context, |ui| {
                        ui.vertical_centered(|ui| ui.heading("bottom panel"))
                    });
                egui::TopBottomPanel::top("top panel")
                    .resizable(false)
                    .default_height(50.0)
                    .show(&context, |ui| {
                        ui.vertical_centered(|ui| ui.heading("controls"))
                    });

                let output = context.end_frame();

                state.paint_jobs = context.tessellate(output.shapes);
                state.egui_textures_delta = output.textures_delta;

                barrier.wait();
                render_thread.is_finished();
            }
            Event::MainEventsCleared => {
                window.request_redraw();
            }
            Event::WindowEvent {
                ref event,
                window_id,
            } if window_id == window.id() => {
                // winit_events.push(event);
                // TODO! use for seeing if egui wanted this event or not
                // let res = egui_state.on_event(&context, &event);
                // if !res.consumed {
                //     input(&mut game_renderer, event);
                // }
                // match event {
                //     #[cfg(not(target_arch = "wasm32"))]
                //     WindowEvent::CloseRequested
                //     | WindowEvent::KeyboardInput {
                //         input:
                //             KeyboardInput {
                //                 state: ElementState::Pressed,
                //                 virtual_keycode: Some(VirtualKeyCode::Escape),
                //                 ..
                //             },
                //         ..
                //     } => {
                //         halt = true},
                //     winit::event::WindowEvent::Resized(size) => {
                //         // Resize with 0 width and height is used by winit to signal a minimize event on Windows.
                //         // See: https://github.com/rust-windowing/winit/issues/208
                //         // This solves an issue where the app would panic when minimizing on Windows.
                //         if size.width > 0 && size.height > 0 {
                //             config.width = size.width;
                //             config.height = size.height;
                //             surface.configure(&device, &config);
                //             game_renderer.resize(*size, &config);
                //         }
                //     }

                //     _ => {winit_events.push(event.clone())}
                // }
            }
            // Event::DeviceEvent {
            //     event: DeviceEvent::MouseMotion{ delta, },
            //     .. // We're not using device_id currently
            // } => if game_renderer.mouse_pressed {
            //     game_renderer.cameras[game_renderer.active_camera.unwrap() as usize].controller.process_mouse(delta.0, delta.1)
            // }
            e => winit_events.push(e.to_static().clone()),
        }
    });
}
