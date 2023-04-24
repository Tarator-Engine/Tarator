use std::f32::consts::PI;

use tar_types::{Mat4, Vec4};
use wgpu::util::DeviceExt;
use winit::window::Window;

use crate::{model::Model, state::RenderState};
use tar_shader::shader::{
    self,
    bind_groups::{BindGroup0, BindGroupLayout0},
};

#[rustfmt::skip]
pub const OPENGL_TO_WGPU_MATRIX: Mat4 = glam::Mat4::from_cols_array(&[
    1.0, 0.0, 0.0, 0.0,
    0.0, 1.0, 0.0, 0.0,
    0.0, 0.0, 0.5, 0.0,
    0.0, 0.0, 0.5, 1.0,
]);

pub async fn new_state(window: Window) -> RenderState {
    let size = window.inner_size();

    // The instance is a handle to our GPU
    // Backends::all => Vulkan + Metal + DX12 + Browser WebGPU
    let instance = wgpu::Instance::new(wgpu::InstanceDescriptor {
        backends: wgpu::Backends::all(),
        dx12_shader_compiler: Default::default(),
    });

    // # Safety
    //
    // The surface needs to live as long as the window that created it.
    // State owns the window so this should be safe.
    let surface = unsafe { instance.create_surface(&window) }.unwrap();

    let adapter = instance
        .request_adapter(&wgpu::RequestAdapterOptions {
            power_preference: wgpu::PowerPreference::default(),
            compatible_surface: Some(&surface),
            force_fallback_adapter: true,
        })
        .await
        .unwrap();

    let (device, queue) = adapter
        .request_device(
            &wgpu::DeviceDescriptor {
                features: wgpu::Features::empty(),
                limits: if cfg!(target_arch = "wasm32") {
                    wgpu::Limits::downlevel_webgl2_defaults()
                } else {
                    wgpu::Limits::default()
                },
                label: None,
            },
            None, // Trace path
        )
        .await
        .unwrap();

    let surface_caps = surface.get_capabilities(&adapter);
    // Shader code in this tutorial assumes an sRGB surface texture. Using a different
    // one will result all the colors coming out darker. If you want to support non
    // sRGB surfaces, you'll need to account for that when drawing to the frame.
    let surface_format = surface_caps
        .formats
        .iter()
        .copied()
        .filter(|f| f.describe().srgb)
        .next()
        .unwrap_or(surface_caps.formats[0]);
    let config = wgpu::SurfaceConfiguration {
        usage: wgpu::TextureUsages::RENDER_ATTACHMENT,
        format: surface_format,
        width: size.width,
        height: size.height,
        present_mode: wgpu::PresentMode::AutoNoVsync, // TODO!: is there some easy way to make this user configurable
        alpha_mode: surface_caps.alpha_modes[0],
        view_formats: vec![],
    };
    surface.configure(&device, &config);

    let view = glam::Mat4::look_at_rh(
        (0.0, 1.0, 2.0).into(),
        (0.0, 0.0, 0.0).into(),
        glam::Vec3::Y,
    );

    let proj = glam::Mat4::perspective_rh(
        PI / 2.0,
        config.width as f32 / config.height as f32,
        0.1,
        100.0,
    );

    // TODO!: move at least the object transform part to somewhere in the object
    let uniform_data = shader::UniformData {
        ambient: Vec4::new(0.2, 0.3, 0.5, 1.0),
        view: view.into(),
        view_proj: (OPENGL_TO_WGPU_MATRIX * proj * view).into(),
        object_transform: glam::Mat4::from_cols_array_2d(&[[0.0; 4]; 4]),
    };

    let mut uni_buff = encase::UniformBuffer::new(vec![]);

    uni_buff.write(&uniform_data).unwrap();

    let uniform_data_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
        label: Some("object uniform buffer"),
        contents: &uni_buff.into_inner(),
        usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
    });

    let primary_sampler = device.create_sampler(&wgpu::SamplerDescriptor {
        address_mode_u: wgpu::AddressMode::ClampToEdge,
        address_mode_v: wgpu::AddressMode::ClampToEdge,
        address_mode_w: wgpu::AddressMode::ClampToEdge,
        mag_filter: wgpu::FilterMode::Linear,
        min_filter: wgpu::FilterMode::Nearest,
        mipmap_filter: wgpu::FilterMode::Nearest,
        ..Default::default()
    });

    let light_data = vec![shader::DirectionalLight {
        color: [0.5, 0.5, 0.0].into(),
        padding: 0.0,
        direction: [0.5, 0.5, 0.5].into(),
        padding2: 0.0,
    }];

    let mut light_buffer = encase::StorageBuffer::new(vec![]);
    light_buffer.write(&light_data).unwrap();

    let light_storage_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
        label: Some("light storage buffer"),
        contents: &light_buffer.into_inner(),
        usage: wgpu::BufferUsages::STORAGE | wgpu::BufferUsages::COPY_DST,
    });

    let global_frame_bind_group = BindGroup0::from_bindings(
        &device,
        BindGroupLayout0 {
            primary_sampler: &primary_sampler,
            uniforms: uniform_data_buffer.as_entire_buffer_binding(),
            directional_lights: light_storage_buffer.as_entire_buffer_binding(),
        },
    );

    let models = tar_res::import_models("assets/scifi_helmet/SciFiHelmet.gltf").unwrap();

    let models = models
        .into_iter()
        .map(|model| Model::from_stored(model, &device, &queue, config.format))
        .collect();

    println!("inited state");

    RenderState {
        window,
        surface,
        device,
        queue,
        config,
        size,
        global_frame_bind_group,
        models,
    }
}

pub fn resize(new_size: winit::dpi::PhysicalSize<u32>, state: &mut RenderState) {
    if new_size.width > 0 && new_size.height > 0 {
        state.size = new_size;
        state.config.width = new_size.width;
        state.config.height = new_size.height;
        state.surface.configure(&state.device, &state.config);
    }
}

pub fn render(state: &mut RenderState) -> Result<(), wgpu::SurfaceError> {
    let output = state.surface.get_current_texture()?;
    let view = output
        .texture
        .create_view(&wgpu::TextureViewDescriptor::default());
    let mut encoder = state
        .device
        .create_command_encoder(&wgpu::CommandEncoderDescriptor {
            label: Some("Render Encoder"),
        });

    {
        let mut render_pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
            label: Some("Render Pass"),
            color_attachments: &[
                // This is what @location(0) in the fragment shader targets
                Some(wgpu::RenderPassColorAttachment {
                    view: &view,
                    resolve_target: None,
                    ops: wgpu::Operations {
                        load: wgpu::LoadOp::Clear(wgpu::Color {
                            r: 0.1,
                            g: 0.2,
                            b: 0.3,
                            a: 1.0,
                        }),
                        store: true,
                    },
                }),
            ],
            depth_stencil_attachment: None,
        });

        state.global_frame_bind_group.set(&mut render_pass);

        for model in &state.models {
            model.render(&mut render_pass);
        }
    }

    state.queue.submit(std::iter::once(encoder.finish()));
    output.present();

    Ok(())
}
