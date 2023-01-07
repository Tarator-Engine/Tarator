use std::sync::Arc;

use egui_wgpu::renderer::ScreenDescriptor;
use winit::{
    event::*,
    event_loop::{ControlFlow, EventLoop},
};

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
        .with_fullscreen(Some(winit::window::Fullscreen::Borderless(None)))
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
    let mut config = wgpu::SurfaceConfiguration {
        usage: wgpu::TextureUsages::RENDER_ATTACHMENT,
        format: surface.get_supported_formats(&adapter)[0],
        width: size.width,
        height: size.height,
        present_mode: wgpu::PresentMode::AutoNoVsync,
        alpha_mode: surface.get_supported_alpha_modes(&adapter)[0],
    };
    surface.configure(&device, &config);

    let mut game_renderer = tar_render::render::forward::ForwardRenderer::new(
        device.clone(),
        queue.clone(),
        &config,
        config.format,
    )
    .await;

    let int_camera = tar_render::camera::IntCamera::new(
        (0.0, 5.0, 10.0),
        cgmath::Deg(-90.0),
        cgmath::Deg(-20.0),
    );
    let projection = tar_render::camera::Projection::new(
        config.width,
        config.height,
        cgmath::Deg(45.0),
        0.1,
        100.0,
    );
    let camera_controller = tar_render::camera::CameraController::new(4.0, 0.4);

    let camera = tar_render::camera::Camera {
        cam: int_camera,
        proj: projection,
        controller: camera_controller,
    };

    game_renderer
        .add_object(tar_render::GameObject::ModelPath("res/Box/Box.gltf", "box"))
        .await
        .unwrap();

    game_renderer
        .add_object(tar_render::GameObject::Camera(camera))
        .await
        .unwrap();
    game_renderer.select_camera(0);

    let mut egui_renderer =
        egui_wgpu::Renderer::new(&device, surface.get_supported_formats(&adapter)[0], None, 1);
    let mut egui_state = egui_winit::State::new(&event_loop);

    let context = egui::Context::default();

    let mut frames = 0;
    let mut fps = 0;
    let mut since_start = 0;
    let start_time = instant::Instant::now();

    let mut last_render_time = start_time;
    event_loop.run(move |event, _, control_flow| {
        match event {
            Event::RedrawRequested(..) => {
                let now = instant::Instant::now();
                let dt = now - last_render_time;
                last_render_time = now;
                update(&mut game_renderer, dt);
                let secs = start_time.elapsed().as_secs();
                if secs > since_start {
                    since_start = secs;
                    fps = frames;
                    frames = 0;
                }


                let output_frame = match surface.get_current_texture() {
                    Ok(frame) => frame,
                    Err(wgpu::SurfaceError::Outdated) => {
                        // This error occurs when the app is minimized on Windows.
                        // Silently return here to prevent spamming the console with:
                        // "The underlying surface has changed, and therefore the swap chain must be updated"
                        return;
                    }
                    Err(e) => {
                        eprintln!("Dropped frame with error: {}", e);
                        return;
                    }
                };
                let view = output_frame
                    .texture
                    .create_view(&wgpu::TextureViewDescriptor::default());

                let input = egui_state.take_egui_input(&window);
                context.begin_frame(input);
                egui::Window::new("Timings").show(&context, |ui| {
                    ui.label("Here you can see different frame timings");
                    ui.label(format!("Frame time: {dt:?}"));
                    ui.label(format!("FPS: {fps}"));
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
                        ui.add(egui::Slider::new(&mut game_renderer.cameras[game_renderer.active_camera.unwrap() as usize].controller.sensitivity, 0.0..=5.0));
                    });
                egui::TopBottomPanel::bottom("bottom panel").resizable(true).default_height(200.0).show(&context, |ui| {
                    ui.vertical_centered(|ui| ui.heading("bottom panel"))
                });
                egui::TopBottomPanel::top("top panel").resizable(false).default_height(50.0).show(&context, |ui| {
                    ui.vertical_centered(|ui| ui.heading("controls"))
                });

                let output = context.end_frame();
                let paint_jobs = context.tessellate(output.shapes);

                let mut encoder = device.create_command_encoder(&wgpu::CommandEncoderDescriptor {
                    label: Some("encoder"),
                });

                // My rendering
                {
                    // Blah blah render pipeline stuff here

                    match game_renderer.render(&mut encoder, &view) {
                        Ok(_) => {}
                        // Reconfigure the surface if it's lost or outdated
                        Err(wgpu::SurfaceError::Lost | wgpu::SurfaceError::Outdated) => {
                            game_renderer.resize(size, &config);
                        }
                        // The system is out of memory, we should probably quit
                        Err(wgpu::SurfaceError::OutOfMemory) => *control_flow = ControlFlow::Exit,
                        // We're ignoring timeouts
                        Err(wgpu::SurfaceError::Timeout) => log::warn!("Surface timeout"),
                    }
                }

                // Egui rendering now
                let screen_descriptor = ScreenDescriptor {
                    size_in_pixels: [config.width, config.height],
                    // Forcing pixels per point 1.0 - the egui input handling seems to not scale the cursor coordinates automatically
                    pixels_per_point: 1.0,
                };

                let user_cmd_bufs = {
                    for (id, image_delta) in &output.textures_delta.set {
                        egui_renderer.update_texture(&device, &queue, *id, image_delta);
                    }

                    egui_renderer.update_buffers(
                        &device,
                        &queue,
                        &mut encoder,
                        &paint_jobs.as_ref(),
                        &screen_descriptor,
                    )
                };

                egui_renderer.update_buffers(
                    &device,
                    &queue,
                    &mut encoder,
                    &paint_jobs.as_ref(),
                    &screen_descriptor,
                );
                {
                    let mut render_pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
                        label: Some("UI Render Pass"),
                        color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                            view: &view,
                            resolve_target: None,
                            ops: wgpu::Operations {
                                load: wgpu::LoadOp::Load,
                                store: true,
                            },
                        })],
                        depth_stencil_attachment: None,
                    });

                    egui_renderer.render(
                        &mut render_pass,
                        &paint_jobs.as_ref(),
                        &screen_descriptor,
                    );
                }

                for id in &output.textures_delta.free {
                    egui_renderer.free_texture(id);
                }

                queue.submit(
                    user_cmd_bufs
                        .into_iter()
                );
                queue.submit(std::iter::once(encoder.finish()));

                // Redraw egui
                output_frame.present();
                frames += 1;
            }
            Event::MainEventsCleared => {
                window.request_redraw();
            }
            Event::WindowEvent {
                ref event,
                window_id,
            } if window_id == window.id() => {
                // TODO use for seeing if egui wanted this event or not
                let res = egui_state.on_event(&context, &event);
                if !res.consumed {
                    input(&mut game_renderer, event);
                }
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
                    } => *control_flow = ControlFlow::Exit,
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
