use std::{collections::HashMap, f32::consts::PI};

use scr_types::{
    prims::{Mat4, Vec3, Vec4},
    RenderEntities,
};
use wgpu::util::DeviceExt;
use winit::window::Window;

use crate::{camera, model::texture, state::RenderState};
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

pub async fn new_state(window: &Window) -> RenderState {
    let size = window.inner_size();

    // The instance is a handle to our GPU
    // Backends::all => Vulkan + Metal + DX12 + Browser WebGPU
    let backends = if cfg!(windows) {
        wgpu::Backends::DX12
    } else {
        wgpu::Backends::all()
    };

    dbg!(backends);
    let instance = wgpu::Instance::new(wgpu::InstanceDescriptor {
        backends,
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
            force_fallback_adapter: false, // whatever you do, do not set this to true it tanks performance instantly
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
        .find(|f| f.describe().srgb)
        .unwrap_or(surface_caps.formats[0]);

    dbg!(surface_caps.formats);
    dbg!(surface_format);
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
        (2.0, 2.0, 2.0).into(),
        (0.0, 0.0, 0.0).into(),
        glam::Vec3::Y,
    );

    let proj = glam::Mat4::perspective_rh(
        std::f32::consts::FRAC_PI_4,
        config.width as f32 / config.height as f32,
        0.1,
        100.0,
    );

    let uniform_data = shader::UniformData {
        ambient: Vec4::new(0.2, 0.3, 0.5, 1.0),
        view,
        proj,
        camera_pos: Vec4::splat(0.0),
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
        color: [23.47, 21.31, 20.79].into(),
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
    // let box_models = tar_res::import_models("assets/box/Box.gltf").unwrap();

    // let mut box_models = box_models
    //     .into_iter()
    //     .map(|model| Model::from_stored(model, &device, &queue, config.format))
    //     .collect();

    // let schifi_models = tar_res::import_models("assets/scifi_helmet/SciFiHelmet.gltf").unwrap();

    // let mut models: Vec<Model> = schifi_models
    //     .into_iter()
    //     .map(|model| Model::from_stored(model, &device, &queue, config.format))
    //     .collect();

    // models.append(&mut box_models);

    let editor_cam = camera::Camera::new((2.0, 2.0, 2.0), -PI / 4.0 * 3.0, -PI / 4.0);
    let editor_cam_controller = camera::CameraController::new(1.0, 1.0);
    let editor_projection =
        camera::Projection::new(config.width, config.height, 60.0, 0.01, 1000.0);

    let depth_tex = texture::DepthTexture::create_depth_texture(&device, &config);

    let size = wgpu::Extent3d {
        width: config.width,
        height: config.height,
        depth_or_array_layers: 1,
    };

    let render_target_tex = device.create_texture(&wgpu::TextureDescriptor {
        label: Some("render target texture"),
        size,
        mip_level_count: 1,
        sample_count: 1,
        dimension: wgpu::TextureDimension::D2,
        format: wgpu::TextureFormat::Rgba8UnormSrgb,
        usage: wgpu::TextureUsages::RENDER_ATTACHMENT | wgpu::TextureUsages::TEXTURE_BINDING,
        view_formats: &[],
    });
    let render_target_tex_view =
        render_target_tex.create_view(&wgpu::TextureViewDescriptor::default());

    RenderState {
        surface,
        queue,
        device,
        adapter,
        config,
        global_frame_bind_group,
        models: vec![],
        uniform_buffer: uniform_data_buffer,
        uniform_data,
        editor_cam,
        editor_cam_controller,
        editor_projection,
        mouse_pressed: false,
        depth_tex,

        render_target_tex,
        render_target_tex_view,
    }
}

pub fn resize(new_size: winit::dpi::PhysicalSize<u32>, state: &mut RenderState) {
    if new_size.width > 0 && new_size.height > 0 {
        state.config.width = new_size.width;
        state.config.height = new_size.height;
        state.surface.configure(&state.device, &state.config);
        state.depth_tex = texture::DepthTexture::create_depth_texture(&state.device, &state.config);
    }
}

pub fn render(
    state: &mut RenderState,
    encoder: &mut wgpu::CommandEncoder,
    surface_view: &wgpu::TextureView,
    dt: std::time::Duration,
    entities: RenderEntities,
) -> Result<(), wgpu::SurfaceError> {
    state
        .editor_cam_controller
        .update_camera(&mut state.editor_cam, dt);

    let view = calc_view_matrix(&state.editor_cam);
    let proj = calc_proj_matrix(&state.editor_projection);

    state.uniform_data.view = view;
    state.uniform_data.proj = proj;
    state.uniform_data.camera_pos = Vec4::from((state.editor_cam.position, 0.0));

    let mut uni_buff = encase::UniformBuffer::new(vec![]);

    uni_buff.write(&state.uniform_data).unwrap();

    state
        .queue
        .write_buffer(&state.uniform_buffer, 0, &uni_buff.into_inner());

    {
        let mut render_pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
            label: Some("Render Pass"),
            color_attachments: &[
                // This is what @location(0) in the fragment shader targets
                Some(wgpu::RenderPassColorAttachment {
                    view: surface_view,
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
            depth_stencil_attachment: Some(wgpu::RenderPassDepthStencilAttachment {
                view: &state.depth_tex.view,
                depth_ops: Some(wgpu::Operations {
                    load: wgpu::LoadOp::Clear(1.0),
                    store: true,
                }),
                stencil_ops: None,
            }),
        });

        state.global_frame_bind_group.set(&mut render_pass);

        for model in &mut state.models {
            if let Some((t, _)) = entities.entities.iter().find(|e| e.1.model_id == *model.0) {
                model.1.update_transform(t, &state.queue);
                model.1.render(&mut render_pass);
            }
        }
    }
    Ok(())
}

fn calc_view_matrix(cam: &camera::Camera) -> Mat4 {
    let (sin_pitch, cos_pitch) = cam.pitch.sin_cos();
    let (sin_yaw, cos_yaw) = cam.yaw.sin_cos();

    Mat4::look_to_rh(
        cam.position.into(),
        Vec3::new(cos_pitch * cos_yaw, sin_pitch, cos_pitch * sin_yaw)
            .normalize()
            .into(),
        Vec3::Y.into(),
    )
}

fn calc_proj_matrix(projection: &camera::Projection) -> Mat4 {
    Mat4::perspective_rh(
        projection.fovy * PI / 180.0,
        projection.aspect,
        projection.znear,
        projection.zfar,
    )
}
