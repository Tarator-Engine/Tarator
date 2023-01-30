use std::sync::{Arc, Barrier};

use egui_wgpu::renderer::ScreenDescriptor;
use parking_lot::{Mutex, RwLock};
use winit::window::Window;

use crate::{error_msg, DoubleBuffer, EngineState};

pub fn render_fn(
    window: Arc<Window>,
    state: Arc<Mutex<DoubleBuffer<EngineState>>>,
    r_barrier: Arc<Barrier>,
    mut egui_state: Arc<egui_winit::State>,
    mut game_renderer: Arc<tar_render::render::forward::ForwardRenderer>,
    mut egui_renderer: Arc<egui_wgpu::Renderer>,
    config: Arc<wgpu::SurfaceConfiguration>,
    surface: Arc<wgpu::Surface>,
    device: Arc<wgpu::Device>,
    queue: Arc<wgpu::Queue>,
    size: Arc<winit::dpi::PhysicalSize<u32>>,
    render_s: Arc<RwLock<bool>>,
) {
    let mut errors: Vec<Box<dyn std::error::Error>> = vec![];

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
        .add_object(tar_render::GameObject::ImportedPath("assets/helmet.rmp"))
        .unwrap();

    game_renderer
        .add_object(tar_render::GameObject::Camera(camera))
        .unwrap();
    game_renderer.select_camera(0);

    let context = egui::Context::default();
    loop {
        if let Some(_) = render_s.read().then_some(true) {
            return;
        }

        r_barrier.wait();
        let state = state.lock().update_read();

        // do rendering here

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

        let mut remove = vec![];
        for (i, err) in (&errors).iter().enumerate() {
            if error_msg::error_message(&context, err) {
                remove.push(i);
            };
        }
        for r in remove.iter().rev() {
            errors.remove(*r);
        }

        egui::Window::new("Timings")
            .resizable(false)
            .show(&context, |ui| {
                ui.label("Here you can see different frame timings");
                ui.label(format!("Frame time: {:?}", state.dt));
                ui.label(format!("FPS: {}", state.fps));
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
                ui.label("sensitvity");
                ui.add(egui::Slider::new(
                    &mut game_renderer.cameras[game_renderer.active_camera.unwrap() as usize]
                        .controller
                        .sensitivity,
                    0.0..=5.0,
                ));
            });
        egui::TopBottomPanel::bottom("bottom panel")
            .resizable(true)
            .default_height(200.0)
            .show(&context, |ui| {
                ui.vertical_centered(|ui| ui.heading("bottom panel"))
            });
        egui::TopBottomPanel::top("top panel")
            .resizable(false)
            .default_height(50.0)
            .show(&context, |ui| {
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
                    game_renderer.resize(*size, &config);
                }
                // The system is out of memory, we should probably quit
                Err(wgpu::SurfaceError::OutOfMemory) => {
                    return; // TODO!: return error to clarify
                }
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

            egui_renderer.render(&mut render_pass, &paint_jobs.as_ref(), &screen_descriptor);
        }

        for id in &output.textures_delta.free {
            egui_renderer.free_texture(id);
        }

        queue.submit(user_cmd_bufs.into_iter());
        queue.submit(std::iter::once(encoder.finish()));

        // Redraw egui
        output_frame.present();
    }
}
