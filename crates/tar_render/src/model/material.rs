use bitflags::bitflags;
use image::DynamicImage;
use tar_shader::shader::{
    self,
    bind_groups::{BindGroup1, BindGroupLayout1},
    MaterialData,
};
use wgpu::util::DeviceExt;

use super::texture::{GrayTexture, RgbaTexture, Texture};

pub struct Material {
    pub pbr: PbrMaterial,
    pub normal: Option<NormalMap>,
    pub occlusion: Option<Occlusion>,
    pub emissive: Emissive,
    pub pipeline: wgpu::RenderPipeline,
    pub bind_group: BindGroup1,
    pub material_data_buffer: wgpu::Buffer,
}
impl Material {
    pub fn from_stored(
        stored: tar_res::model::material::Material,
        device: &wgpu::Device,
        queue: &wgpu::Queue,
        target_format: wgpu::TextureFormat,
    ) -> Self {
        // TODO!: make this not a constant
        let mat_flags = MaterialFlags::FLAGS_ALBEDO_ACTIVE;

        let mut tex_flags = TextureFlags::empty();

        tex_flags |= if stored.pbr.base_color_texture.is_some() {
            TextureFlags::TEXTURE_ALBEDO
        } else {
            TextureFlags::empty()
        };

        tex_flags |= if stored.emissive.texture.is_some() {
            TextureFlags::TEXTURE_EMISSIVE
        } else {
            TextureFlags::empty()
        };

        tex_flags |= if stored.pbr.metallic_texture.is_some() {
            TextureFlags::TEXTURE_METALLIC
        } else {
            TextureFlags::empty()
        };

        tex_flags |= if stored.normal.is_some() {
            TextureFlags::TEXTURE_NORMAL
        } else {
            TextureFlags::empty()
        };

        // TODO!: make this not a constant
        tex_flags |= if false {
            TextureFlags::TEXTURE_REFLECTANCE
        } else {
            TextureFlags::empty()
        };

        tex_flags |= if stored.pbr.roughness_texture.is_some() {
            TextureFlags::TEXTURE_ROUGHNESS
        } else {
            TextureFlags::empty()
        };

        let mat_data = MaterialData {
            albedo: stored.pbr.base_color_factor.into(),
            emissive: stored.emissive.factor.into(),
            roughness: stored.pbr.roughness_factor,
            metallic: stored.pbr.metallic_factor,
            reflectance: 0.5, // TODO!: figure out some sensible constant
            flags: mat_flags.bits(),
            texture_enable: tex_flags.bits(),
        };

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
            dimension: wgpu::TextureDimension::D2,
            format: wgpu::TextureFormat::Rgba8UnormSrgb,
            usage: wgpu::TextureUsages::TEXTURE_BINDING,
            label: Some("empty texture"),
            view_formats: &[],
        });
        let empty_view = empty_tex.create_view(&wgpu::TextureViewDescriptor::default());

        let mut mat_uniform = encase::UniformBuffer::new(vec![]);
        mat_uniform.write(&mat_data).unwrap();

        let material_data_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("material uniform buffer"),
            contents: &mat_uniform.into_inner(),
            usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
        });

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
                material_uniform: material_data_buffer.as_entire_buffer_binding(),
            },
        );

        Self {
            pbr,
            normal,
            occlusion,
            emissive,
            pipeline,
            material_data_buffer,
            bind_group: bind_group,
        }
    }
}

pub struct PbrMaterial {
    // pub base_color_factor: Vec4,
    pub base_color_texture: Option<RgbaTexture>,
    pub metallic_texture: Option<GrayTexture>,
    // pub metallic_factor: f32,
    pub roughness_texture: Option<GrayTexture>,
    // pub roughness_factor: f32,
}

impl PbrMaterial {
    pub fn from_stored(
        stored: tar_res::model::material::PbrMaterial,
        device: &wgpu::Device,
        queue: &wgpu::Queue,
    ) -> Self {
        Self {
            // base_color_factor: stored.base_color_factor,
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
            // metallic_factor: stored.metallic_factor,
            roughness_texture: stored.roughness_texture.map(|img| {
                GrayTexture::from_image(
                    device,
                    queue,
                    &DynamicImage::ImageLuma8(img.inner),
                    "roughness_texture",
                )
            }),
            // roughness_factor: stored.roughness_factor,
        }
    }
}

pub struct NormalMap {
    pub texture: RgbaTexture,
    // pub factor: f32,
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
            // factor: stored.factor,
        }
    }
}

pub struct Occlusion {
    pub texture: GrayTexture,
    // pub factor: f32,
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
            // factor: stored.factor,
        }
    }
}

pub struct Emissive {
    pub texture: Option<RgbaTexture>,
    // pub factor: Vec3,
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
            // factor: stored.factor,
        }
    }
}

bitflags! {
    #[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
    struct MaterialFlags: u32 {
        const FLAGS_ALBEDO_ACTIVE = 0b00000001;
        const FLAGS_UNLIT =         0b00000010;
    }
}

bitflags! {
    #[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
    struct TextureFlags: u32 {
        const TEXTURE_ALBEDO =      0b00000001;
        const TEXTURE_NORMAL =      0b00000010;
        const TEXTURE_ROUGHNESS =   0b00000100;
        const TEXTURE_METALLIC =    0b00001000;
        const TEXTURE_REFLECTANCE = 0b00010000;
        const TEXTURE_EMISSIVE =    0b00100000;
    }
}
