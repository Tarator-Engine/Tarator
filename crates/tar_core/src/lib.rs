mod error_msg;
mod pre_render;
mod render;
mod state;

use std::sync::{Arc, Barrier};

use instant::Duration;
use parking_lot::Mutex;

use parking_lot::RwLock;
use winit::{
    event::*,
    event_loop::{ControlFlow, EventLoop},
};

#[derive(Debug)]
struct DoubleBuffer<T: Clone> {
    pub state: T,
}

impl<T: Clone> DoubleBuffer<T> {
    pub fn update_read(&mut self) -> T {
        return self.state.clone();
    }
}

#[derive(Debug, Clone)]
struct EngineState {
    dt: Duration,
    fps: u32,
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
    let window = Arc::new(
        winit::window::WindowBuilder::new()
            .with_title(title)
            .with_fullscreen(Some(winit::window::Fullscreen::Borderless(None)))
            .build(&event_loop)
            .unwrap(),
    );

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

    let mut errors: Vec<Box<dyn std::error::Error>> = vec![];

    let blocking_threads = threadpool::ThreadPool::new(4);

    let barrier = Arc::new(Barrier::new(3));

    let stop = Arc::new(RwLock::new(false));

    let (game_renderer, device, queue, size, surface, config, adapter) =
        build_renderer_extras(&window).await;
    let surface = Arc::new(surface);

    let mut egui_renderer =
        egui_wgpu::Renderer::new(&device, surface.get_supported_formats(&adapter)[0], None, 1);
    let mut egui_state = egui_winit::State::new(&event_loop);

    // let pre_render_sync = Arc::new(Barrier::new(2));
    let p_barrier = barrier.clone();
    let pre_render_s = stop.clone();
    let pre_render_thread = std::thread::spawn(|| {
        pre_render::pre_render_fn(pre_render_s, p_barrier);
    });
    let r_barrier = barrier.clone();
    let render_s = stop.clone();
    let w_clone = window.clone();
    let db_clone = db.clone();
    let s_clone = surface.clone();
    let d_clone = device.clone();
    let q_clone = queue.clone();
    let egui_state = Arc::new(egui_state);
    let game_renderer = Arc::new(game_renderer);
    let egui_renderer = Arc::new(egui_renderer);
    let config = Arc::new(config);
    let size = Arc::new(size);
    let render_thread = std::thread::spawn(move || {
        render::render_fn(
            w_clone,
            db_clone,
            r_barrier,
            egui_state,
            game_renderer,
            egui_renderer,
            config,
            s_clone,
            d_clone,
            q_clone,
            size,
            render_s,
        );
    });

    event_loop.run(move |event, _, control_flow| {
        match event {
            Event::RedrawRequested(..) => {
                barrier.wait();
            }
            Event::MainEventsCleared => {
                window.request_redraw();
            }
            Event::WindowEvent {
                ref event,
                window_id,
            } if window_id == window.id() => {
                // TODO use for seeing if egui wanted this event or not
                // let res = egui_state.on_event(&context, &event);
                // if !res.consumed {
                //     input(&mut game_renderer, event);
                // }
                match event {
                    #[cfg(not(target_arch = "wasm32"))]
                    WindowEvent::CloseRequested
                    | WindowEvent::KeyboardInput {
                        input:
                            KeyboardInput {
                                state: ElementState::Pressed,
                                virtual_keycode: Some(VirtualKeyCode::Escape),
                                ..
                            },
                        ..
                    } => {
                        let w = stop.write();
                        *w = true;
                        *control_flow = ControlFlow::Exit;},
                    winit::event::WindowEvent::Resized(size) => {
                        // Resize with 0 width and height is used by winit to signal a minimize event on Windows.
                        // See: https://github.com/rust-windowing/winit/issues/208
                        // This solves an issue where the app would panic when minimizing on Windows.
                        if size.width > 0 && size.height > 0 {
                            config.width = size.width;
                            config.height = size.height;
                            surface.configure(&device, &config);
                            game_renderer.resize(*size, &config);
                        }
                    }

                    _ => {}
                }
            }
            Event::DeviceEvent {
                event: DeviceEvent::MouseMotion{ delta, },
                .. // We're not using device_id currently
            } => if game_renderer.mouse_pressed {
                game_renderer.cameras[game_renderer.active_camera.unwrap() as usize].controller.process_mouse(delta.0, delta.1)
            }
            _ => (),
        }
    });
}
