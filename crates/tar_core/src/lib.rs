mod double_buffer;
mod error_msg;
mod render;
// mod state;

use std::{sync::{Arc, Barrier}, f32::consts::FRAC_2_PI};

use cgmath::{InnerSpace};
use parking_lot::{MutexGuard, Mutex};
use tar_ecs::{prelude::Entity, world::World};
use tar_types::{components::{Transform, Rendering, Camera, Info}, prims::{Vec3}};
use winit::{
    event::{DeviceEvent, ElementState, Event, KeyboardInput, MouseButton, VirtualKeyCode, WindowEvent},
    event_loop::{ControlFlow, EventLoop},
};

use crate::double_buffer::DoubleBuffer;

pub use tar_types::EngineState;

const SAFE_FRAC_PI_2: f32 = FRAC_2_PI - 0.0001;


/// Takes a window event and a renderer and an event
fn input(
    event: &WindowEvent,
    cam: Entity,
    world: &mut World
) {
    let (_, mut cam) = world.entity_get_mut::<(Transform, Camera)>(cam).unwrap().get_mut();
    match event {
        WindowEvent::KeyboardInput {
            input:
                KeyboardInput {
                    virtual_keycode: Some(key),
                    state,
                    ..
                },
            ..
        } => {
            

            let amount = if *state == ElementState::Pressed {
                1.0
            } else {
                0.0
            };
            match key {
                VirtualKeyCode::W | VirtualKeyCode::Up => {
                    cam.amount_forward = amount;
                    
                }
                VirtualKeyCode::S | VirtualKeyCode::Down => {
                    cam.amount_backward = amount;
                    
                }
                VirtualKeyCode::A | VirtualKeyCode::Left => {
                    cam.amount_left = amount;
                    
                }
                VirtualKeyCode::D | VirtualKeyCode::Right => {
                    cam.amount_right = amount;
                    
                }
                VirtualKeyCode::Space => {
                    cam.amount_up = amount;
                }
                VirtualKeyCode::LShift => {
                    cam.amount_down = amount;
                }
                _ => (),
            }
    
        },
        WindowEvent::MouseInput {
            button: MouseButton::Left,
            state,
            ..
        } => {
            cam.mouse_pressed = *state == ElementState::Pressed;
        }
        _ => (),
    }
}

fn update(
    cam: Entity,
    world: &mut World,
    dt: std::time::Duration,
) {
    let (mut t, mut cam) = world.entity_get_mut::<(Transform, Camera)>(cam).unwrap().get_mut();
    // UPDATE CAMERA
    let dt = dt.as_secs_f32();

    // Move forward/backward and left/right
    let (yaw_sin, yaw_cos) = t.rot.y.0.sin_cos();
    let forward = Vec3::new(yaw_cos, 0.0, yaw_sin).normalize();
    let right = Vec3::new(-yaw_sin, 0.0, yaw_cos).normalize();
    t.pos += forward * (cam.amount_forward - cam.amount_backward) * cam.speed * dt;
    t.pos += right * (cam.amount_right - cam.amount_left) * cam.speed * dt;

    // Move up/down. Since we don't use roll, we can just
    // modify the y coordinate directly.
    t.pos.y += (cam.amount_up - cam.amount_down) * cam.speed * dt;

    // Rotate
    t.rot.y += cgmath::Rad(cam.rotate_horizontal) * cam.sensitivity * dt;
    t.rot.x += cgmath::Rad(-cam.rotate_vertical) * cam.sensitivity * dt;

    // If process_mouse isn't called every frame, these values
    // will not get set to zero, and the camera will rotate
    // when moving in a non cardinal direction.
    cam.rotate_horizontal = 0.0;
    cam.rotate_vertical = 0.0;

    // Keep the camera's angle from going too high/low.
    if t.rot.x < -cgmath::Rad(SAFE_FRAC_PI_2) {
        t.rot.x = -cgmath::Rad(SAFE_FRAC_PI_2);
    } else if t.rot.x > cgmath::Rad(SAFE_FRAC_PI_2) {
        t.rot.x = cgmath::Rad(SAFE_FRAC_PI_2);
    }

    // not necessary I think 
    // TODO!: find out if this is needed
    // UPDATE VIEW PROJECTION MATRIX
    // cam.uniform.update_view_proj(&cam.cam, &cam.proj);
}


async fn build_renderer_extras(
    window: &winit::window::Window,
) -> (
    tar_render::render::forward::ForwardRenderer,
    Arc<wgpu::Device>,
    Arc<wgpu::Queue>,
    wgpu::Surface,
    wgpu::SurfaceConfiguration,
    wgpu::Adapter,
) {
    let desc = wgpu::InstanceDescriptor {
        backends: wgpu::Backends::PRIMARY,
        dx12_shader_compiler: wgpu::Dx12Compiler::Fxc,
    };
    let instance = wgpu::Instance::new(desc);
    let surface = unsafe { instance.create_surface(&window).unwrap() };

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
        format: surface.get_capabilities(&adapter).formats[0],
        width: size.width,
        height: size.height,
        present_mode: wgpu::PresentMode::AutoNoVsync,
        alpha_mode: surface.get_capabilities(&adapter).alpha_modes[0],
        view_formats: vec![],
    };
    surface.configure(&device, &config);

    let game_renderer = tar_render::render::forward::ForwardRenderer::new(
        device.clone(),
        queue.clone(),
        &config,
        config.format,
    )
    .await;

    (game_renderer, device, queue, surface, config, adapter)
}

pub async fn run() {
    let db = DoubleBuffer::new(EngineState::default());
    let db = Arc::new(db);

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
        // .with_fullscreen(Some(winit::window::Fullscreen::Borderless(None)))
        .build(&event_loop)
        .expect("failed to aquire window");

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

    let pre_render_finished = Arc::new(Barrier::new(2));

    let (game_renderer, device, queue, surface, config, adapter) =
        build_renderer_extras(&window).await;

    let surface = Arc::new(surface);

    let egui_renderer = egui_wgpu::Renderer::new(
        &device,
        surface.get_capabilities(&adapter).formats[0],
        None,
        1,
    );

    let mut egui_state = egui_winit::State::new(&event_loop);


    let mut world = tar_ecs::world::World::new();

    let cam = world.entity_create();

    world.entity_set(cam, (Transform::default(), Camera::default()));

    let world = Arc::new(Mutex::new(world));
    

    let r_barrier = pre_render_finished.clone();
    let s_clone = surface;
    let d_clone = device;
    let q_clone = queue;
    let engine_state = db.clone();
    let w_clone = world.clone();
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
            w_clone,
        );
    });
    let mut errors: Vec<Box<dyn std::error::Error>> = vec![];

    let context = egui::Context::default();

    let mut winit_events = vec![];

    let mut last_render_time = instant::Instant::now();
    let start_time = last_render_time;
    let mut since_start = 0;
    let mut frames = 0;

    let mut file_dialogue = None;
    
    let mut entities = vec![];

    event_loop.run(move |event, _, control_flow| {
        match event {
            Event::RedrawRequested(..) => {
                let mut state = db.lock();
                let mut world = world.lock();
                // state.events = vec![];
                state.mouse_movement.0 = 0.0;
                state.mouse_movement.1 = 0.0;
                state.add_object = None;

                let now = instant::Instant::now();
                state.dt = now - last_render_time;
                last_render_time = now;
                let secs = start_time.elapsed().as_secs();
                frames += 1;
                if secs > since_start {
                    since_start = secs;
                    state.fps = frames;
                    frames = 0;
                }

                // flatten skips all instances where the event is None
                for event in winit_events.iter().flatten() {
                    let e: &Event<()> = event;
                    match e {
                        Event::WindowEvent {
                            ref event,
                            window_id,
                        } if window_id == &window.id() => {

                            let _res = egui_state.on_event(&context, event);

                            input(event, cam, &mut world);

                            // if state.mouse_in_view || !res.consumed {
                            //     state.events.push(event.clone().to_static().unwrap());
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
                                } => state.halt = true,
                                winit::event::WindowEvent::Resized(size) => {
                                    // Resize with 0 width and height is used by winit to signal a minimize event on Windows.
                                    // See: https://github.com/rust-windowing/winit/issues/208
                                    // This solves an issue where the app would panic when minimizing on Windows.
                                    if size.width > 0 && size.height > 0 {
                                        state.size = *size;
                                    }
                                }

                                _ => (),
                            }
                        },
                        Event::DeviceEvent {
                            event: DeviceEvent::MouseMotion{ delta, },
                            .. // We're not using device_id currently
                        } => {
                            state.mouse_movement.0 += delta.0;
                            state.mouse_movement.1 += delta.1;
                            state.mouse_pos.x += delta.0 as f32;
                            state.mouse_pos.y += delta.1 as f32;
                            
                            state.mouse_in_view = state.view_rect.contains(state.mouse_pos);
                        }
                        _ => (),
                    }
                }
                winit_events = vec![];

                {
                    let (_, mut cam) = world.entity_get_mut::<(Transform, Camera)>(cam).unwrap().get_mut();
                    if cam.mouse_pressed {
                        cam.rotate_horizontal = state.mouse_movement.0 as f32;
                        cam.rotate_vertical = state.mouse_movement.1 as f32;
                    }
                }

                {
                    if !entities.is_empty(){
                        let (mut t, _) = world.entity_get_mut::<(Transform, Rendering)>(entities[0]).unwrap().get_mut();
                        t.pos.x = f32::sin(start_time.elapsed().as_secs_f32());
                    }
                }

                update(cam, &mut world, state.dt);

                let input = egui_state.take_egui_input(&window);
                context.begin_frame(input);

                let mut remove = vec![];
                for (i, err) in errors.iter().enumerate() {
                    if error_msg::error_message(&context, err) {
                        remove.push(i);
                    };
                }
                for r in remove.iter().rev() {
                    errors.remove(*r);
                }

                tar_gui::gui(&context, &mut state, &mut file_dialogue, &mut world);

                if let Some((id, _)) = &state.add_object {
                    let e = world.entity_create();
                    world.entity_set(e, (Transform::default(), Rendering {model_id: *id}, Info::default()));
                    entities.push(e);
                }

                let output = context.end_frame();

                state.paint_jobs = context.tessellate(output.shapes);
                state.egui_textures_delta = output.textures_delta;

                if state.halt {
                    *control_flow = ControlFlow::Exit;
                    return;
                }
                MutexGuard::unlock_fair(world);
                MutexGuard::unlock_fair(state);

                if render_thread.is_finished() {
                    print!("error: render thread has crashed");
                }

                pre_render_finished.wait();
            }
            Event::MainEventsCleared => {
                window.request_redraw();
            }
            e => winit_events.push(e.to_static().clone()),
        }
    });
}
