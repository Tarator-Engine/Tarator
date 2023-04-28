use std::sync::{Arc, Barrier};

use egui_wgpu::renderer::ScreenDescriptor;
use parking_lot::Mutex;
use tar_render::render_functions;

use crate::{state::ShareState, DoubleBuffer};

pub struct RenderData {
    pre_render_finished_barrier: Arc<Barrier>,
    shared_state: Arc<Mutex<DoubleBuffer<ShareState>>>,
    game_render_state: tar_render::state::RenderState,
    egui_renderer: egui_wgpu::Renderer,
}

pub fn render_fn(mut data: RenderData) {
    let RenderData {
        pre_render_finished_barrier,
        shared_state,
        game_render_state,
        egui_renderer,
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
                render_functions::resize(game_render_state.size, &mut game_render_state);
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

            // My rendering
            {
                let rendered_objects = components.iter().map(|(_, c)| c.model_id).collect();
                game_renderer.render(&mut encoder, &view, rendered_objects);
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
