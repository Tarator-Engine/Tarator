use std::sync::{Arc, Barrier};

use egui_wgpu::renderer::ScreenDescriptor;
use parking_lot::Mutex;
use tar_render::render_functions;

use crate::{state::ShareState, DoubleBuffer};

pub struct RenderData {
    pub pre_render_finished_barrier: Arc<Barrier>,
    pub shared_state: Arc<Mutex<DoubleBuffer<ShareState>>>,
    pub game_render_state: tar_render::state::RenderState,
    pub egui_renderer: egui_wgpu::Renderer,
}

pub fn render_fn(data: RenderData) {
    let RenderData {
        pre_render_finished_barrier,
        shared_state,
        mut game_render_state,
        mut egui_renderer,
    } = data;

    loop {
        pre_render_finished_barrier.wait();
        let shared_state = shared_state.lock().update_read();

        // do rendering here

        if shared_state.halt {
            return;
        }

        let output_frame = match game_render_state.surface.get_current_texture() {
            Ok(frame) => Some(frame),
            Err(wgpu::SurfaceError::Lost | wgpu::SurfaceError::Outdated) => {
                render_functions::resize(shared_state.window_size, &mut game_render_state);
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

            let mut encoder =
                game_render_state
                    .device
                    .create_command_encoder(&wgpu::CommandEncoderDescriptor {
                        label: Some("encoder"),
                    });

            // My rendering
            render_functions::render(&mut game_render_state, &mut encoder, shared_state.dt)
                .unwrap();

            println!("my_render done");

            // Egui rendering now
            let screen_descriptor = ScreenDescriptor {
                size_in_pixels: [
                    shared_state.window_size.width,
                    shared_state.window_size.height,
                ],
                // Forcing pixels per point 1.0 - the egui input handling seems to not scale the cursor coordinates automatically
                pixels_per_point: 1.0,
            };

            let user_cmd_bufs = {
                for (id, image_delta) in &shared_state.egui_textures_delta.set {
                    egui_renderer.update_texture(
                        &game_render_state.device,
                        &game_render_state.queue,
                        *id,
                        image_delta,
                    );
                }

                egui_renderer.update_buffers(
                    &game_render_state.device,
                    &game_render_state.queue,
                    &mut encoder,
                    &shared_state.paint_jobs.as_ref(),
                    &screen_descriptor,
                )
            };

            egui_renderer.update_buffers(
                &game_render_state.device,
                &game_render_state.queue,
                &mut encoder,
                &shared_state.paint_jobs.as_ref(),
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
                    &shared_state.paint_jobs.as_ref(),
                    &screen_descriptor,
                );
            }
            println!("egui_render done");

            for id in &shared_state.egui_textures_delta.free {
                egui_renderer.free_texture(id);
            }

            game_render_state.queue.submit(user_cmd_bufs.into_iter());
            game_render_state
                .queue
                .submit(std::iter::once(encoder.finish()));
            output_frame.present();
        }
    }
}
