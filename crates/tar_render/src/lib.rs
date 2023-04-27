// use glam::Vec4Swizzles;
// use tar_shader::shader;
// use tar_types::{Mat3, Mat4, Vec3, Vec4};

mod camera;
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

        let mut render_state = new_state(window).await;

        let mut last_render_time = std::time::Instant::now();

        event_loop.run(move |event, _, control_flow| match event {
            Event::RedrawRequested(window_id) if window_id == render_state.window.id() => {
                let now = std::time::Instant::now();
                render_state.dt = now - last_render_time;
                last_render_time = now;
                match render(&mut render_state) {
                    Ok(_) => {}
                    // Reconfigure the surface if lost
                    Err(wgpu::SurfaceError::Lost) => resize(render_state.size, &mut render_state),
                    // The system is out of memory, we should probably quit
                    Err(wgpu::SurfaceError::OutOfMemory) => *control_flow = ControlFlow::Exit,
                    // All other errors (Outdated, Timeout) should be resolved by the next frame
                    Err(e) => eprintln!("{:?}", e),
                }
            }
            Event::MainEventsCleared => {
                // RedrawRequested will only trigger once, unless we manually
                // request it.
                render_state.window.request_redraw();
            }
            Event::DeviceEvent {
                event: DeviceEvent::MouseMotion{ delta, },
                .. // We're not using device_id currently
            } => if render_state.mouse_pressed {
                render_state.editor_cam_controller.process_mouse(delta.0, delta.1)
            }
            Event::WindowEvent {
                ref event,
                window_id,
            } if window_id == render_state.window.id() => match event {
                WindowEvent::KeyboardInput {
                    input:
                        KeyboardInput {
                            virtual_keycode: Some(key),
                            state,
                            ..
                        },
                    ..
                } => {
                    render_state
                        .editor_cam_controller
                        .process_keyboard(*key, *state);
                }
                WindowEvent::MouseWheel { delta, .. } => {
                    render_state.editor_cam_controller.process_scroll(delta);
                }
                WindowEvent::MouseInput {
                    button: MouseButton::Left,
                    state,
                    ..
                } => {
                    render_state.mouse_pressed = *state == ElementState::Pressed;
                }
                WindowEvent::CloseRequested => *control_flow = ControlFlow::Exit,
                WindowEvent::Resized(physical_size) => {
                    resize(*physical_size, &mut render_state);
                }
                WindowEvent::ScaleFactorChanged { new_inner_size, .. } => {
                    // new_inner_size is &&mut so we have to dereference it twice
                    resize(**new_inner_size, &mut render_state);
                }
                _ => {}
            },
            _ => {}
        });
    }
}

// pub fn vs_analouge(vertex: &shader::Vertex) {
//     let normal: Vec3 = vertex.normal.into();
//     let tangent: Vec3 = vertex.tangent.xyz().into();
//     let view = glam::Mat4::look_at_rh(
//         (2.0, 2.0, 2.0).into(),
//         (0.0, 0.0, 0.0).into(),
//         glam::Vec3::Y,
//     );

//     #[rustfmt::skip]
//     let proj =
//         glam::Mat4::perspective_rh(
//             std::f32::consts::PI / 2.0,
//             1920.0 / 1080.0,
//             0.1,
//             100.0);

//     let model_matrix = Mat4::IDENTITY;
//     let model_view = view * model_matrix;
//     let model_view_proj = proj * model_view;

//     let position_vec4 = Vec4::from((vertex.position, 1.0));

//     // let view_pos = model_view * position_vec4;
//     // dbg!(model_matrix);
//     // dbg!(view);
//     // dbg!(model_view);
//     // dbg!(view_pos);

//     // let mvp = proj * view * model_matrix;

//     // let homogeneous = mvp * position_vec4;
//     // dbg!(homogeneous);

//     // dbg!(view);

//     // let world_pos = model_matrix * position_vec4;
//     // dbg!(world_pos);
//     // let camera_pos = view * world_pos;
//     // dbg!(camera_pos);
//     // let homogeneous = proj * camera_pos;
//     // dbg!(homogeneous);

//     let mv_mat3 = Mat3::from_cols(
//         model_view.x_axis.truncate(),
//         model_view.y_axis.truncate(),
//         model_view.z_axis.truncate(),
//     );

//     let inv_scale_sq = mat3_inv_scale_squared(mv_mat3);

//     // dbg!(inv_scale_sq);

//     let view_position = model_view * position_vec4;
//     // dbg!(view_position);
//     let normal = (mv_mat3 * (inv_scale_sq * normal)).normalize();
//     // dbg!(normal);
//     let tangent = (mv_mat3 * (inv_scale_sq * tangent)).normalize();
//     let tangent = Vec4::new(tangent.x, tangent.y, tangent.z, 1.0);
//     // dbg!(tangent);
//     let position = model_view_proj * position_vec4;
//     dbg!(position);
// }

// fn mat3_inv_scale_squared(matrix: Mat3) -> Vec3 {
//     Vec3::new(
//         1.0 / glam::Vec3::dot(matrix.x_axis, matrix.x_axis),
//         1.0 / glam::Vec3::dot(matrix.y_axis, matrix.y_axis),
//         1.0 / glam::Vec3::dot(matrix.z_axis, matrix.z_axis),
//     )
// }
