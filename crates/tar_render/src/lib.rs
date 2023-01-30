pub mod camera;
pub mod render;

use render::deferred::DeferredRenderer;
use winit::{
    event::*,
    event_loop::{ControlFlow, EventLoop},
};

#[cfg(target_arch = "wasm32")]
use wasm_bindgen::prelude::*;

use crate::render::Renderer;

/// the GameObject is a enum which is used to pass something to a
/// Renderer's add_object()
pub enum GameObject<'a> {
    Object(tar_res::object::Object),
    ModelPath(&'a str, &'a str),
    ImportedPath(&'a str),
    Camera(camera::Camera),
}

fn input(renderer: &mut DeferredRenderer, event: &WindowEvent) -> bool {
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

fn update(renderer: &mut DeferredRenderer, dt: std::time::Duration) {
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

    let mut renderer = render::deferred::DeferredRenderer::new(&window).await;

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

    // const NUM_INSTANCES_PER_ROW: u32 = 10;
    // const INSTANCE_DISPLACEMENT: cgmath::Vector3<f32> = cgmath::Vector3::new(
    //     NUM_INSTANCES_PER_ROW as f32 * 0.5,
    //     0.0,
    //     NUM_INSTANCES_PER_ROW as f32 * 0.5,
    // );

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
    renderer.select_camera(0);

    let mut last_render_time = instant::Instant::now();
    event_loop.run(move |event, _, control_flow| {
        *control_flow = ControlFlow::Poll;
        match event {
            Event::MainEventsCleared => window.request_redraw(),
            Event::DeviceEvent {
                event: DeviceEvent::MouseMotion{ delta, },
                .. // We're not using device_id currently
            } => if renderer.mouse_pressed {
                renderer.cameras[renderer.active_camera.unwrap() as usize].controller.process_mouse(delta.0, delta.1)
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
