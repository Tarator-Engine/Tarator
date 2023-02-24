use std::sync::{Arc, Barrier};

use egui_wgpu::renderer::ScreenDescriptor;
use parking_lot::Mutex;
use tar_types::{
    components::{Camera, Rendering, Transform},
    prims::Quat,
};

use crate::{DoubleBuffer, EngineState};

// /// Takes a window event and a renderer and an event
// fn input(
//     renderer: &mut tar_render::render::forward::ForwardRenderer,
//     event: &WindowEvent,
//     cam: &uuid::Uuid,
// ) -> bool {
//     match event {
//         WindowEvent::KeyboardInput {
//             input:
//                 KeyboardInput {
//                     virtual_keycode: Some(key),
//                     state,
//                     ..
//                 },
//             ..
//         } => renderer
//             .cameras
//             .get_mut(cam)
//             .unwrap()
//             .controller
//             .process_keyboard(*key, *state),
//         WindowEvent::MouseWheel { delta, .. } => {
//             renderer
//                 .cameras
//                 .get_mut(cam)
//                 .unwrap()
//                 .controller
//                 .process_scroll(delta);
//             true
//         }
//         WindowEvent::MouseInput {
//             button: MouseButton::Left,
//             state,
//             ..
//         } => {
//             renderer.mouse_pressed = *state == ElementState::Pressed;
//             true
//         }

//         _ => false,
//     }
// }

// fn update(
//     renderer: &mut tar_render::render::forward::ForwardRenderer,
//     dt: std::time::Duration,
//     cam: &uuid::Uuid,
// ) {
//     let cam = &mut renderer.cameras.get_mut(cam).unwrap();
//     cam.controller.update_camera(&mut cam.cam, dt);
//     cam.uniform.update_view_proj(&cam.cam, &cam.proj);
// }

pub fn render_fn(
    r_barrier: Arc<Barrier>,
    engine_state: Arc<DoubleBuffer<EngineState>>,
    mut game_renderer: tar_render::render::forward::ForwardRenderer,
    mut egui_renderer: egui_wgpu::Renderer,
    mut config: wgpu::SurfaceConfiguration,
    surface: Arc<wgpu::Surface>,
    device: Arc<wgpu::Device>,
    queue: Arc<wgpu::Queue>,
    world: Arc<Mutex<tar_ecs::world::World>>,
) {
    // let int_camera = tar_render::camera::IntCamera::new(
    //     (0.0, 5.0, 10.0),
    //     cgmath::Deg(-90.0),
    //     cgmath::Deg(-20.0),
    // );
    // let projection = tar_render::camera::Projection::new(
    //     config.width,
    //     config.height,
    //     cgmath::Deg(45.0),
    //     0.1,
    //     100.0,
    // );
    // let camera_controller = tar_render::camera::CameraController::new(4.0, 0.4);

    // let camera = tar_render::camera::Camera {
    //     cam: int_camera,
    //     proj: projection,
    //     controller: camera_controller,
    // };

    // let test_id =
    //     game_renderer.add_object(tar_render::GameObject::ImportedPath("assets/helmet.rmp"));

    // let cam = game_renderer.add_camera(camera);
    // game_renderer.select_camera(cam);

    let mut loaded_objects = vec![];

    loop {
        r_barrier.wait();
        let state = engine_state.lock().update_read();
        let objects_state = world.lock().component_collect::<(Transform, Rendering)>();
        let cameras_state = world.lock().component_collect::<(Transform, Camera)>();

        // do rendering here
        for obj in &loaded_objects {
            game_renderer.check_done(*obj).unwrap();
        }

        for (t, r) in &objects_state {
            if let Some(obj) = game_renderer.objects.get_mut(&r.model_id) {
                //TODO!: implementation for multiple nodes
                obj.nodes[0].translation = t.pos;
                obj.nodes[0].rotation = Quat::from(t.rot);
                obj.nodes[0].scale = t.scale;
            }
        }

        if let Some((id, path)) = state.add_object {
            game_renderer.add_object(tar_render::GameObject::ImportedPath(&path), id);
            loaded_objects.push(id);
        }

        if state.halt {
            return;
        }

        // for event in state.events {
        //     input(&mut game_renderer, &event, &cam);
        // }

        // if game_renderer.mouse_pressed {
        //     game_renderer
        //         .cameras
        //         .get_mut(&cam)
        //         .unwrap()
        //         .controller
        //         .process_mouse(state.mouse_movement.0, state.mouse_movement.1);
        // }
        // game_renderer
        //     .cameras
        //     .get_mut(&cam)
        //     .unwrap()
        //     .controller
        //     .sensitivity = state.cam_sensitivity;

        // update(&mut game_renderer, state.dt, &cam);

        let output_frame = match surface.get_current_texture() {
            Ok(frame) => Some(frame),
            Err(wgpu::SurfaceError::Lost | wgpu::SurfaceError::Outdated) => {
                game_renderer.resize(state.size, &mut config);
                surface.configure(&device, &config);
                None
            }
            Err(wgpu::SurfaceError::Timeout) => {
                eprintln!("Surface timeout");
                None
            }
            Err(wgpu::SurfaceError::OutOfMemory) => return,
        };
        if let Some(output_frame) = output_frame {
            let view = output_frame
                .texture
                .create_view(&wgpu::TextureViewDescriptor::default());

            // rendering

            let mut encoder = device.create_command_encoder(&wgpu::CommandEncoderDescriptor {
                label: Some("encoder"),
            });

            // Game rendering
            if let Some(cam) = cameras_state.into_iter().find(|cam| cam.1.active) {
                match game_renderer.render(&mut encoder, &view, objects_state, cam, state.size) {
                    Ok(()) => (),
                    Err(e) => eprintln!("Rendering failed with error: {e:?}"),
                }
            }

            // Egui rendering now
            let screen_descriptor = ScreenDescriptor {
                size_in_pixels: [config.width, config.height],
                // Forcing pixels per point 1.0 - the egui input handling seems to not scale the cursor coordinates automatically
                pixels_per_point: 1.0,
            };

            let user_cmd_bufs = {
                for (id, image_delta) in &state.egui_textures_delta.set {
                    egui_renderer.update_texture(&device, &queue, *id, image_delta);
                }

                egui_renderer.update_buffers(
                    &device,
                    &queue,
                    &mut encoder,
                    &state.paint_jobs.as_ref(),
                    &screen_descriptor,
                )
            };

            egui_renderer.update_buffers(
                &device,
                &queue,
                &mut encoder,
                &state.paint_jobs.as_ref(),
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
                    &state.paint_jobs.as_ref(),
                    &screen_descriptor,
                );
            }

            for id in &state.egui_textures_delta.free {
                egui_renderer.free_texture(id);
            }

            queue.submit(user_cmd_bufs.into_iter());
            queue.submit(std::iter::once(encoder.finish()));
            output_frame.present();
        }
    }
}
