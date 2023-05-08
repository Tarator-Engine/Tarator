mod add_object;
mod convert_event;
mod double_buffer;
mod render;
mod state;

use std::sync::{Arc, Barrier};

use convert_event::deref_event;
use parking_lot::{Mutex, MutexGuard};
use tar_render::render_functions;
use winit::{
    event::{
        DeviceEvent, ElementState, Event, KeyboardInput, MouseButton, VirtualKeyCode, WindowEvent,
    },
    event_loop::{ControlFlow, EventLoop},
};

use crate::double_buffer::DoubleBuffer;

use tar_gui::GuiInData;

pub async fn run() {
    let db = DoubleBuffer::new(state::ShareState::default());
    let share_state = Arc::new(Mutex::new(db));

    let event_loop = EventLoop::new();
    let window = winit::window::WindowBuilder::new()
        .with_title("Tarator")
        // .with_fullscreen(Some(winit::window::Fullscreen::Borderless(None)))
        .build(&event_loop)
        .expect("failed to aquire window");

    let game_render_state = render_functions::new_state(&window).await;

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

    let egui_renderer = egui_wgpu::Renderer::new(
        &game_render_state.device,
        game_render_state.config.format,
        None,
        1,
    );

    let mut egui_state = egui_winit::State::new(&event_loop);

    let render_data = render::RenderData {
        pre_render_finished_barrier: pre_render_finished.clone(),
        shared_state: share_state.clone(),
        game_render_state,
        egui_renderer,
    };

    std::thread::spawn(move || {
        render::render_fn(render_data);
    });

    let context = egui::Context::default();

    let mut winit_device_events: Vec<DeviceEvent> = vec![];
    let mut winit_window_events: Vec<WindowEvent> = vec![];

    let mut last_render_time = instant::Instant::now();
    let start_time = last_render_time;
    let mut since_start = 0;
    let mut frames: u32 = 0;

    let mut main_thread_state = state::MainThreadState {
        window,
        scripts_lib: None,
        scripts_systems: None,
    };

    let mut gui_data = GuiInData::default();

    event_loop.run(move |event, _, control_flow| {
        match event {
            Event::RedrawRequested(..) => {
                let mut share_state = share_state.lock();
                share_state.resize = false;
                let now = instant::Instant::now();
                share_state.dt = now - last_render_time;
                gui_data.dt = share_state.dt;
                last_render_time = now;
                let secs = start_time.elapsed().as_secs();
                frames += 1;
                if secs > since_start {
                    since_start = secs;
                    gui_data.fps = frames;
                    frames = 0;
                }

                for event in &winit_window_events {
                    let _res = egui_state.on_event(&context, &event);

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
                        } => share_state.halt = true,
                        winit::event::WindowEvent::Resized(size) => {
                            // Resize with 0 width and height is used by winit to signal a minimize event on Windows.
                            // See: https://github.com/rust-windowing/winit/issues/208
                            // This solves an issue where the app would panic when minimizing on Windows.
                            if size.width > 0 && size.height > 0 {
                                share_state.window_size = *size;
                                share_state.resize = true;
                            }
                        }
                        WindowEvent::MouseInput {
                            state,
                            button: MouseButton::Left,
                            ..
                        } => {
                            if *state == ElementState::Pressed {
                                share_state.mouse_pressed = true;
                            } else {
                                share_state.mouse_pressed = false;
                            }
                        }

                        _ => (),
                    }
                }
                winit_window_events = vec![];
                share_state.device_events = winit_device_events.clone();
                winit_device_events = vec![];

                let input = egui_state.take_egui_input(&main_thread_state.window);
                context.begin_frame(input);

                let gui_out = tar_gui::gui(&context, &mut gui_data);

                share_state.mouse_in_view = gui_out.mouse_in_game_view;

                // run scripts
                if gui_data.running {
                    if let Some(scr_lib) = &main_thread_state.scripts_lib {
                        tar_abi::run_scripts(
                            scr_lib,
                            main_thread_state.scripts_systems.as_ref().unwrap(),
                        );
                    }
                }
                if gui_out.reload_scripts {
                    main_thread_state.scripts_lib = None;
                    main_thread_state.scripts_systems = None;
                    let (lib, systems) = tar_abi::load_scripts_lib().unwrap();
                    main_thread_state.scripts_lib = Some(lib);
                    main_thread_state.scripts_systems = Some(systems);
                }

                let output = context.end_frame();

                share_state.paint_jobs = context.tessellate(output.shapes);
                share_state.egui_textures_delta = output.textures_delta;

                if share_state.halt {
                    *control_flow = ControlFlow::Exit;
                }
                MutexGuard::unlock_fair(share_state);

                // if *(&render_thread.is_finished()) {
                //     eprintln!("error: render thread has crashed");
                // }

                pre_render_finished.wait();
            }
            Event::MainEventsCleared => {
                main_thread_state.window.request_redraw();
            }
            e => match e {
                Event::WindowEvent {
                    window_id: _,
                    event,
                } => winit_window_events.push(deref_event(&event)),

                Event::DeviceEvent {
                    device_id: _,
                    event,
                } => winit_device_events.push(event),
                _ => (),
            },
        }
    });
}
