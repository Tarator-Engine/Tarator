use image::DynamicImage;
use tar_shader::shader::{
    self,
    bind_groups::{BindGroup1, BindGroupLayout1},
};
use tar_types::{Vec3, Vec4};

use super::texture::{GrayTexture, RgbaTexture, Texture};

pub struct Material {
    pub pbr: PbrMaterial,
    pub normal: Option<NormalMap>,
    pub occlusion: Option<Occlusion>,
    pub emissive: Emissive,
    pub pipeline: wgpu::RenderPipeline,
    pub bind_group: BindGroup1,
}
impl Material {
    pub fn from_stored(
        stored: tar_res::model::material::Material,
        device: &wgpu::Device,
        queue: &wgpu::Queue,
        target_format: wgpu::TextureFormat,
    ) -> Self {
        let shader = shader::create_shader_module(device);

        let pipeline_layout = shader::create_pipeline_layout(device);

        let pipeline = device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
            label: Some("internal render pipeline"),
            layout: Some(&pipeline_layout),
            vertex: wgpu::VertexState {
                module: &shader,
                entry_point: "vs_main",
                buffers: &[shader::Vertex::vertex_buffer_layout(
                    wgpu::VertexStepMode::Vertex,
                )],
            },
            fragment: Some(wgpu::FragmentState {
                module: &shader,
                entry_point: "fs_main",
                targets: &[Some(wgpu::ColorTargetState {
                    format: target_format,
                    blend: Some(wgpu::BlendState::REPLACE),
                    write_mask: wgpu::ColorWrites::ALL,
                })],
            }),
            primitive: wgpu::PrimitiveState {
                topology: wgpu::PrimitiveTopology::TriangleList,
                strip_index_format: None,
                front_face: wgpu::FrontFace::Ccw,
                cull_mode: Some(wgpu::Face::Back),
                // Setting this to anything other than Fill requires Features::NON_FILL_POLYGON_MODE
                polygon_mode: wgpu::PolygonMode::Fill,
                // Requires Features::DEPTH_CLIP_CONTROL
                unclipped_depth: false,
                // Requires Features::CONSERVATIVE_RASTERIZATION
                conservative: false,
            },
            depth_stencil: None,
            multisample: wgpu::MultisampleState {
                count: 1,
                mask: !0,
                alpha_to_coverage_enabled: false,
            },
            multiview: None,
        });
        let mut material_uniform_data = shader::MaterialData {
            albedo: [0.0; 4],
            emissive: [0.0; 3],
            roughness: 0.0,
            metallic: 0.0,
            reflectance: 0.0,
            flags: 0,
            texture_enable: 0,
        };

        let pbr = PbrMaterial::from_stored(stored.pbr, device, queue);
        let normal = stored
            .normal
            .map(|n| NormalMap::from_stored(n, device, queue));
        let occlusion = stored
            .occlusion
            .map(|o| Occlusion::from_stored(o, device, queue));
        let emissive = Emissive::from_stored(stored.emissive, device, queue);

        let empty_tex = device.create_texture(&wgpu::TextureDescriptor {
            size: wgpu::Extent3d {
                width: 1,
                height: 1,
                depth_or_array_layers: 1,
            },
            mip_level_count: 1,
            sample_count: 1,
            dimension: wgpu::TextureDimension::D1,
            format: wgpu::TextureFormat::Rgba8UnormSrgb,
            usage: wgpu::TextureUsages::empty(),
            label: Some("empty texture"),
            view_formats: &[],
        });
        let empty_view = empty_tex.create_view(&wgpu::TextureViewDescriptor::default());

        let bind_group = BindGroup1::from_bindings(
            device,
            BindGroupLayout1 {
                roughness_tex: pbr
                    .roughness_texture
                    .as_ref()
                    .map(|t| &t.view)
                    .unwrap_or(&empty_view),
                normal_tex: normal
                    .as_ref()
                    .map(|t| &t.texture.view)
                    .unwrap_or(&empty_view),
                emissive_tex: emissive
                    .texture
                    .as_ref()
                    .map(|t| &t.view)
                    .unwrap_or(&empty_view),
                albedo_tex: pbr
                    .base_color_texture
                    .as_ref()
                    .map(|t| &t.view)
                    .unwrap_or(&empty_view),
                metallic_tex: pbr
                    .metallic_texture
                    .as_ref()
                    .map(|t| &t.view)
                    .unwrap_or(&empty_view),
            },
        );

        Self {
            pbr,
            normal,
            occlusion,
            emissive,
            pipeline,
        }
    }
}

pub struct PbrMaterial {
    pub base_color_factor: Vec4,
    pub base_color_texture: Option<RgbaTexture>,
    pub metallic_texture: Option<GrayTexture>,
    pub metallic_factor: f32,
    pub roughness_texture: Option<GrayTexture>,
    pub roughness_factor: f32,
}

impl PbrMaterial {
    pub fn from_stored(
        stored: tar_res::model::material::PbrMaterial,
        device: &wgpu::Device,
        queue: &wgpu::Queue,
    ) -> Self {
        Self {
            base_color_factor: stored.base_color_factor,
            base_color_texture: stored.base_color_texture.map(|img| {
                RgbaTexture::from_image(
                    device,
                    queue,
                    &DynamicImage::ImageRgba8(img.inner),
                    "base_color_texture",
                )
            }),
            metallic_texture: stored.metallic_texture.map(|img| {
                GrayTexture::from_image(
                    device,
                    queue,
                    &DynamicImage::ImageLuma8(img.inner),
                    "metallic_texture",
                )
            }),
            metallic_factor: stored.metallic_factor,
            roughness_texture: stored.roughness_texture.map(|img| {
                GrayTexture::from_image(
                    device,
                    queue,
                    &DynamicImage::ImageLuma8(img.inner),
                    "roughness_texture",
                )
            }),
            roughness_factor: stored.roughness_factor,
        }
    }
}

pub struct NormalMap {
    pub texture: RgbaTexture,
    pub factor: f32,
}

impl NormalMap {
    pub fn from_stored(
        stored: tar_res::model::material::NormalMap,
        device: &wgpu::Device,
        queue: &wgpu::Queue,
    ) -> Self {
        Self {
            texture: RgbaTexture::from_image(
                device,
                queue,
                &DynamicImage::ImageRgb8(stored.texture.inner),
                "normal_texture",
            ),
            factor: stored.factor,
        }
    }
}

pub struct Occlusion {
    pub texture: GrayTexture,
    pub factor: f32,
}

impl Occlusion {
    pub fn from_stored(
        stored: tar_res::model::material::Occlusion,
        device: &wgpu::Device,
        queue: &wgpu::Queue,
    ) -> Self {
        Self {
            texture: GrayTexture::from_image(
                device,
                queue,
                &DynamicImage::ImageLuma8(stored.texture.inner),
                "occlusion_texture",
            ),
            factor: stored.factor,
        }
    }
}

pub struct Emissive {
    pub texture: Option<RgbaTexture>,
    pub factor: Vec3,
}

impl Emissive {
    pub fn from_stored(
        stored: tar_res::model::material::Emissive,
        device: &wgpu::Device,
        queue: &wgpu::Queue,
    ) -> Self {
        Self {
            texture: stored.texture.map(|img| {
                RgbaTexture::from_image(
                    device,
                    queue,
                    &DynamicImage::ImageRgb8(img.inner),
                    "emissive_texture",
                )
            }),
            factor: stored.factor,
        }
    }
}
