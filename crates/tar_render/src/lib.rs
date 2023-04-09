pub mod model;
pub mod render_functions;
pub mod state;

pub mod dev {
    use winit::{
        event::*,
        event_loop::{ControlFlow, EventLoop},
        window::WindowBuilder,
    };

    use crate::render_functions::{new_state, render, resize};

    pub async fn run() {
        let event_loop = EventLoop::new();
        let window = WindowBuilder::new().build(&event_loop).unwrap();

        let mut state = new_state(window).await;

        event_loop.run(move |event, _, control_flow| match event {
            Event::RedrawRequested(window_id) if window_id == state.window.id() => {
                match render(&mut state) {
                    Ok(_) => {}
                    // Reconfigure the surface if lost
                    Err(wgpu::SurfaceError::Lost) => resize(state.size, &mut state),
                    // The system is out of memory, we should probably quit
                    Err(wgpu::SurfaceError::OutOfMemory) => *control_flow = ControlFlow::Exit,
                    // All other errors (Outdated, Timeout) should be resolved by the next frame
                    Err(e) => eprintln!("{:?}", e),
                }
            }
            Event::MainEventsCleared => {
                // RedrawRequested will only trigger once, unless we manually
                // request it.
                state.window.request_redraw();
            }
            Event::WindowEvent {
                ref event,
                window_id,
            } if window_id == state.window.id() => match event {
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
                    resize(*physical_size, &mut state);
                }
                WindowEvent::ScaleFactorChanged { new_inner_size, .. } => {
                    // new_inner_size is &&mut so we have to dereference it twice
                    resize(**new_inner_size, &mut state);
                }
                _ => {}
            },
            _ => {}
        });
    }
}
