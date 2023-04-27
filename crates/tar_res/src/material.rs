use std::sync::Arc;

use cgmath::{Vector3, Vector4};
use wgpu::{BindGroupLayoutEntry, ShaderStages};

use crate::{
    shader::{PbrShader, ShaderFlags},
    texture::Texture,
    uniform::Uniform,
    WgpuInfo,
};

pub struct PbrMaterial {
    pub index: usize,
    pub name: Option<String>,

    pub base_color_factor: Vector4<f32>,
    pub metallic_factor: f32,
    pub roughness_factor: f32,

    pub normal_scale: Option<f32>,

    pub occlusion_strength: f32,
    pub emissive_factor: Vector3<f32>,

    pub alpha_cutoff: Option<f32>,
    pub alpha_mode: gltf::material::AlphaMode,

    pub double_sided: bool,

    pub pbr_shader: PbrShader,

    pub per_frame_uniforms: PerFrameUniforms,
    pub per_material_uniforms: PerMaterialUniforms,

    pub pipeline: wgpu::RenderPipeline,
}

impl PbrMaterial {
    pub fn update_per_frame(&mut self, data: &PerFrameData, queue: &wgpu::Queue) {
        self.per_frame_uniforms.update(data, queue);
    }

    pub fn shader_flags(
        base_color_texture: bool,
        normal_texture: bool,
        emissive_texture: bool,
        metallic_roughness_texture: bool,
        occlusion_texture: bool,
    ) -> ShaderFlags {
        let mut flags = ShaderFlags::empty();
        if base_color_texture {
            flags |= ShaderFlags::HAS_BASECOLORMAP;
        }
        if normal_texture {
            flags |= ShaderFlags::HAS_NORMALMAP;
        }
        if emissive_texture {
            flags |= ShaderFlags::HAS_EMISSIVEMAP;
        }
        if metallic_roughness_texture {
            flags |= ShaderFlags::HAS_METALROUGHNESSMAP;
        }
        if occlusion_texture {
            flags |= ShaderFlags::HAS_OCCLUSIONMAP;
        }
        flags
    }

    pub fn set_pipeline<'a, 'b>(&'a self, render_pass: &'b mut wgpu::RenderPass<'a>) {
        render_pass.set_pipeline(&self.pipeline);
    }

    pub fn set_bind_groups<'a, 'b>(&'a self, render_pass: &'b mut wgpu::RenderPass<'a>) {
        render_pass.set_bind_group(0, &self.per_frame_uniforms.bind_group, &[]);
        render_pass.set_bind_group(
            1,
            &self.per_material_uniforms.bind_group.as_ref().unwrap(),
            &[],
        );
    }
}

pub trait BindGroup {
    type Data;
    fn new(data: Self::Data, layout: &wgpu::BindGroupLayout, w_info: Arc<WgpuInfo>) -> Self;
    fn bind_group_layout() -> wgpu::BindGroupLayoutDescriptor<'static>;
    fn names() -> Vec<(String, String)>;
    fn update(&mut self, dat: &Self::Data, queue: &wgpu::Queue);
}

pub struct PerMaterialUniforms {
    pub base_color_texture: Option<Texture>,
    pub metallic_roughness_texture: Option<Texture>,
    pub normal_texture: Option<Texture>,
    pub occlusion_texture: Option<Texture>,
    pub emissive_texture: Option<Texture>,
    pub bind_group: Option<wgpu::BindGroup>,
}

impl PerMaterialUniforms {
    pub fn entries(&self) -> Vec<BindGroupLayoutEntry> {
        let mut entries = vec![];
        let mut binding = 0;
        fn get_binding(binding: &mut u32) -> u32 {
            *binding += 1;
            return *binding - 1;
        }
        if self.base_color_texture.is_some() {
            entries.push(wgpu::BindGroupLayoutEntry {
                binding: get_binding(&mut binding),
                visibility: ShaderStages::FRAGMENT,
                ty: wgpu::BindingType::Texture {
                    sample_type: wgpu::TextureSampleType::Float { filterable: true },
                    view_dimension: wgpu::TextureViewDimension::D2,
                    multisampled: false,
                },
                count: None,
            });
            entries.push(wgpu::BindGroupLayoutEntry {
                binding: get_binding(&mut binding),
                visibility: ShaderStages::FRAGMENT,
                ty: wgpu::BindingType::Sampler(wgpu::SamplerBindingType::Filtering),
                count: None,
            });
        }
        if self.metallic_roughness_texture.is_some() {
            entries.push(wgpu::BindGroupLayoutEntry {
                binding: get_binding(&mut binding),
                visibility: ShaderStages::FRAGMENT,
                ty: wgpu::BindingType::Texture {
                    sample_type: wgpu::TextureSampleType::Float { filterable: true },
                    view_dimension: wgpu::TextureViewDimension::D2,
                    multisampled: false,
                },
                count: None,
            });
            entries.push(wgpu::BindGroupLayoutEntry {
                binding: get_binding(&mut binding),
                visibility: ShaderStages::FRAGMENT,
                ty: wgpu::BindingType::Sampler(wgpu::SamplerBindingType::Filtering),
                count: None,
            });
        }

        if self.normal_texture.is_some() {
            entries.push(wgpu::BindGroupLayoutEntry {
                binding: get_binding(&mut binding),
                visibility: ShaderStages::FRAGMENT,
                ty: wgpu::BindingType::Texture {
                    sample_type: wgpu::TextureSampleType::Float { filterable: true },
                    view_dimension: wgpu::TextureViewDimension::D2,
                    multisampled: false,
                },
                count: None,
            });
            entries.push(wgpu::BindGroupLayoutEntry {
                binding: get_binding(&mut binding),
                visibility: ShaderStages::FRAGMENT,
                ty: wgpu::BindingType::Sampler(wgpu::SamplerBindingType::Filtering),
                count: None,
            });
        }
        if self.occlusion_texture.is_some() {
            entries.push(wgpu::BindGroupLayoutEntry {
                binding: get_binding(&mut binding),
                visibility: ShaderStages::FRAGMENT,
                ty: wgpu::BindingType::Texture {
                    sample_type: wgpu::TextureSampleType::Float { filterable: true },
                    view_dimension: wgpu::TextureViewDimension::D2,
                    multisampled: false,
                },
                count: None,
            });
            entries.push(wgpu::BindGroupLayoutEntry {
                binding: get_binding(&mut binding),
                visibility: ShaderStages::FRAGMENT,
                ty: wgpu::BindingType::Sampler(wgpu::SamplerBindingType::Filtering),
                count: None,
            });
        }
        if self.emissive_texture.is_some() {
            entries.push(wgpu::BindGroupLayoutEntry {
                binding: get_binding(&mut binding),
                visibility: ShaderStages::FRAGMENT,
                ty: wgpu::BindingType::Texture {
                    sample_type: wgpu::TextureSampleType::Float { filterable: true },
                    view_dimension: wgpu::TextureViewDimension::D2,
                    multisampled: false,
                },
                count: None,
            });
            entries.push(wgpu::BindGroupLayoutEntry {
                binding: get_binding(&mut binding),
                visibility: ShaderStages::FRAGMENT,
                ty: wgpu::BindingType::Sampler(wgpu::SamplerBindingType::Filtering),
                count: None,
            });
        }
        entries
    }

    pub fn bind_group_layout<'a>(
        entries: &'a Vec<BindGroupLayoutEntry>,
    ) -> wgpu::BindGroupLayoutDescriptor<'a> {
        wgpu::BindGroupLayoutDescriptor {
            label: Some("per material bind group layout"),
            entries: entries.as_slice(),
        }
    }

    pub fn names(&self) -> Vec<(String, String)> {
        let mut names = vec![];
        if self.base_color_texture.is_some() {
            names.push(("base_color_tex".into(), "texture_2d<f32>".into()));
            names.push(("base_color_sampler".into(), "sampler".into()));
        }
        if self.metallic_roughness_texture.is_some() {
            names.push(("metallic_roughness_tex".into(), "texture_2d<f32>".into()));
            names.push(("metallic_roughness_sampler".into(), "sampler".into()));
        }
        if self.normal_texture.is_some() {
            names.push(("normal_tex".into(), "texture_2d<f32>".into()));
            names.push(("normal_sampler".into(), "sampler".into()));
        }
        if self.occlusion_texture.is_some() {
            names.push(("occlusion_tex".into(), "texture_2d<f32>".into()));
            names.push(("occlusion_sampler".into(), "sampler".into()));
        }
        if self.emissive_texture.is_some() {
            names.push(("emissive_tex".into(), "texture_2d<f32>".into()));
            names.push(("emissive_sampler".into(), "sampler".into()));
        }

        names
    }

    pub fn set_bind_group(&mut self, device: &wgpu::Device, layout: &wgpu::BindGroupLayout) {
        let mut entries = vec![];
        let mut binding = 0;
        fn get_binding(binding: &mut u32) -> u32 {
            *binding += 1;
            return *binding - 1;
        }
        if self.base_color_texture.is_some() {
            entries.push(wgpu::BindGroupEntry {
                binding: get_binding(&mut binding),
                resource: wgpu::BindingResource::TextureView(
                    &self.base_color_texture.as_ref().unwrap().view,
                ),
            });
            entries.push(wgpu::BindGroupEntry {
                binding: get_binding(&mut binding),
                resource: wgpu::BindingResource::Sampler(
                    &self.base_color_texture.as_ref().unwrap().sampler,
                ),
            });
        }
        if self.metallic_roughness_texture.is_some() {
            entries.push(wgpu::BindGroupEntry {
                binding: get_binding(&mut binding),
                resource: wgpu::BindingResource::TextureView(
                    &self.metallic_roughness_texture.as_ref().unwrap().view,
                ),
            });
            entries.push(wgpu::BindGroupEntry {
                binding: get_binding(&mut binding),
                resource: wgpu::BindingResource::Sampler(
                    &self.metallic_roughness_texture.as_ref().unwrap().sampler,
                ),
            });
        }

        if self.normal_texture.is_some() {
            entries.push(wgpu::BindGroupEntry {
                binding: get_binding(&mut binding),
                resource: wgpu::BindingResource::TextureView(
                    &self.normal_texture.as_ref().unwrap().view,
                ),
            });
            entries.push(wgpu::BindGroupEntry {
                binding: get_binding(&mut binding),
                resource: wgpu::BindingResource::Sampler(
                    &self.normal_texture.as_ref().unwrap().sampler,
                ),
            });
        }
        if self.occlusion_texture.is_some() {
            entries.push(wgpu::BindGroupEntry {
                binding: get_binding(&mut binding),
                resource: wgpu::BindingResource::TextureView(
                    &self.occlusion_texture.as_ref().unwrap().view,
                ),
            });
            entries.push(wgpu::BindGroupEntry {
                binding: get_binding(&mut binding),
                resource: wgpu::BindingResource::Sampler(
                    &self.occlusion_texture.as_ref().unwrap().sampler,
                ),
            });
        }
        if self.emissive_texture.is_some() {
            entries.push(wgpu::BindGroupEntry {
                binding: get_binding(&mut binding),
                resource: wgpu::BindingResource::TextureView(
                    &self.emissive_texture.as_ref().unwrap().view,
                ),
            });
            entries.push(wgpu::BindGroupEntry {
                binding: get_binding(&mut binding),
                resource: wgpu::BindingResource::Sampler(
                    &self.emissive_texture.as_ref().unwrap().sampler,
                ),
            });
        }

        self.bind_group = Some(device.create_bind_group(&wgpu::BindGroupDescriptor {
            layout,
            entries: &entries,
            label: Some("bind_group for textures"),
        }));
    }
}

pub struct PerFrameUniforms {
    u_mpv_matrix: Uniform<[[f32; 4]; 4]>,
    u_model_matrix: Uniform<[[f32; 4]; 4]>,
    u_camera: Uniform<[f32; 3]>,

    u_light_direction: Uniform<[f32; 3]>,
    u_light_color: Uniform<[f32; 3]>,

    u_ambient_light_color: Uniform<[f32; 3]>,
    u_ambient_light_intensity: Uniform<f32>,

    u_alpha_blend: Uniform<f32>,
    u_alpha_cutoff: Uniform<f32>,

    pub bind_group: wgpu::BindGroup,
}
#[derive(Debug, Clone)]
pub struct PerFrameData {
    pub u_mpv_matrix: [[f32; 4]; 4],
    pub u_model_matrix: [[f32; 4]; 4],
    pub u_camera: [f32; 3],

    pub u_light_direction: [f32; 3],
    pub u_light_color: [f32; 3],

    pub u_ambient_light_color: [f32; 3],
    pub u_ambient_light_intensity: f32,

    pub u_alpha_blend: f32,
    pub u_alpha_cutoff: f32,
}

impl Default for PerFrameData {
    fn default() -> Self {
        Self {
            u_mpv_matrix: [[0.0; 4]; 4],
            u_model_matrix: [[0.0; 4]; 4],
            u_camera: [0.0; 3],
            u_light_direction: [0.0; 3],
            u_light_color: [0.0; 3],
            u_ambient_light_color: [0.0; 3],
            u_ambient_light_intensity: 0.0,
            u_alpha_blend: 0.0,
            u_alpha_cutoff: 0.0,
        }
    }
}

impl BindGroup for PerFrameUniforms {
    type Data = PerFrameData;
    fn new(data: PerFrameData, layout: &wgpu::BindGroupLayout, w_info: Arc<WgpuInfo>) -> Self {
        let u_mpv_matrix = Uniform::new(data.u_mpv_matrix, "u_mpv_matrix".into(), w_info.clone());
        let u_model_matrix =
            Uniform::new(data.u_model_matrix, "u_model_matrix".into(), w_info.clone());
        let u_camera = Uniform::new(data.u_camera, "u_camera".into(), w_info.clone());
        let u_light_direction = Uniform::new(
            data.u_light_direction,
            "u_light_direction".into(),
            w_info.clone(),
        );
        let u_light_color =
            Uniform::new(data.u_light_color, "u_light_color".into(), w_info.clone());
        let u_ambient_light_color = Uniform::new(
            data.u_ambient_light_color,
            "u_ambient_light_color".into(),
            w_info.clone(),
        );
        let u_ambient_light_intensity = Uniform::new(
            data.u_ambient_light_intensity,
            "u_ambient_light_intensity".into(),
            w_info.clone(),
        );
        let u_alpha_blend =
            Uniform::new(data.u_alpha_blend, "u_alpha_blend".into(), w_info.clone());
        let u_alpha_cutoff =
            Uniform::new(data.u_alpha_cutoff, "u_alpha_cutoff".into(), w_info.clone());

        let bind_group = w_info.device.create_bind_group(&wgpu::BindGroupDescriptor {
            label: Some("per frame bind group"),
            layout,
            entries: &[
                wgpu::BindGroupEntry {
                    binding: 0,
                    resource: u_mpv_matrix.buff.as_entire_binding(),
                },
                wgpu::BindGroupEntry {
                    binding: 1,
                    resource: u_model_matrix.buff.as_entire_binding(),
                },
                wgpu::BindGroupEntry {
                    binding: 2,
                    resource: u_camera.buff.as_entire_binding(),
                },
                wgpu::BindGroupEntry {
                    binding: 3,
                    resource: u_light_direction.buff.as_entire_binding(),
                },
                wgpu::BindGroupEntry {
                    binding: 4,
                    resource: u_light_color.buff.as_entire_binding(),
                },
                wgpu::BindGroupEntry {
                    binding: 5,
                    resource: u_ambient_light_color.buff.as_entire_binding(),
                },
                wgpu::BindGroupEntry {
                    binding: 6,
                    resource: u_ambient_light_intensity.buff.as_entire_binding(),
                },
                wgpu::BindGroupEntry {
                    binding: 7,
                    resource: u_alpha_blend.buff.as_entire_binding(),
                },
                wgpu::BindGroupEntry {
                    binding: 8,
                    resource: u_alpha_cutoff.buff.as_entire_binding(),
                },
            ],
        });

        Self {
            u_mpv_matrix,
            u_model_matrix,
            u_camera,
            u_light_direction,
            u_light_color,
            u_ambient_light_color,
            u_ambient_light_intensity,
            u_alpha_blend,
            u_alpha_cutoff,
            bind_group,
        }
    }

    fn names() -> Vec<(String, String)> {
        vec![
            ("u_mpv_matrix".into(), "mat4x4<f32>".into()),
            ("u_model_matrix".into(), "mat4x4<f32>".into()),
            ("u_camera".into(), "vec3<f32>".into()),
            ("u_light_direction".into(), "vec3<f32>".into()),
            ("u_light_color".into(), "vec3<f32>".into()),
            ("u_ambient_light_color".into(), "vec3<f32>".into()),
            ("u_ambient_light_intensity".into(), "f32".into()),
            ("u_alpha_blend".into(), "f32".into()),
            ("u_alpha_cutoff".into(), "f32".into()),
        ]
    }

    fn update(&mut self, data: &Self::Data, queue: &wgpu::Queue) {
        self.u_mpv_matrix.update(data.u_mpv_matrix, queue);
        self.u_model_matrix.update(data.u_model_matrix, queue);
        self.u_camera.update(data.u_camera, queue);
        self.u_light_direction.update(data.u_light_direction, queue);
        self.u_light_color.update(data.u_light_color, queue);
        self.u_ambient_light_color
            .update(data.u_ambient_light_color, queue);
        self.u_ambient_light_intensity
            .update(data.u_ambient_light_intensity, queue);
        self.u_alpha_blend.update(data.u_alpha_blend, queue);
        self.u_alpha_cutoff.update(data.u_alpha_cutoff, queue);
    }

    fn bind_group_layout() -> wgpu::BindGroupLayoutDescriptor<'static> {
        wgpu::BindGroupLayoutDescriptor {
            label: Some("Per Frame Data"),
            entries: &[
                // u_mpv_matrix: mat4x4<f32>
                wgpu::BindGroupLayoutEntry {
                    binding: 0,
                    visibility: ShaderStages::VERTEX,
                    ty: wgpu::BindingType::Buffer {
                        ty: wgpu::BufferBindingType::Uniform,
                        has_dynamic_offset: false,
                        min_binding_size: None,
                    },
                    count: None,
                },
                // u_model_matrix: mat4x4<f32>
                wgpu::BindGroupLayoutEntry {
                    binding: 1,
                    visibility: ShaderStages::VERTEX,
                    ty: wgpu::BindingType::Buffer {
                        ty: wgpu::BufferBindingType::Uniform,
                        has_dynamic_offset: false,
                        min_binding_size: None,
                    },
                    count: None,
                },
                // u_camera: vec3<f32>
                wgpu::BindGroupLayoutEntry {
                    binding: 2,
                    visibility: ShaderStages::FRAGMENT,
                    ty: wgpu::BindingType::Buffer {
                        ty: wgpu::BufferBindingType::Uniform,
                        has_dynamic_offset: false,
                        min_binding_size: None,
                    },
                    count: None,
                },
                // u_light_direction: vec3<f32>
                wgpu::BindGroupLayoutEntry {
                    binding: 3,
                    visibility: ShaderStages::FRAGMENT,
                    ty: wgpu::BindingType::Buffer {
                        ty: wgpu::BufferBindingType::Uniform,
                        has_dynamic_offset: false,
                        min_binding_size: None,
                    },
                    count: None,
                },
                // u_light_color: vec3<f32>
                wgpu::BindGroupLayoutEntry {
                    binding: 4,
                    visibility: ShaderStages::FRAGMENT,
                    ty: wgpu::BindingType::Buffer {
                        ty: wgpu::BufferBindingType::Uniform,
                        has_dynamic_offset: false,
                        min_binding_size: None,
                    },
                    count: None,
                },
                // u_ambient_light_color: vec3<f32>
                wgpu::BindGroupLayoutEntry {
                    binding: 5,
                    visibility: ShaderStages::FRAGMENT,
                    ty: wgpu::BindingType::Buffer {
                        ty: wgpu::BufferBindingType::Uniform,
                        has_dynamic_offset: false,
                        min_binding_size: None,
                    },
                    count: None,
                },
                // u_ambient_light_intensity: vec3<f32>
                wgpu::BindGroupLayoutEntry {
                    binding: 6,
                    visibility: ShaderStages::FRAGMENT,
                    ty: wgpu::BindingType::Buffer {
                        ty: wgpu::BufferBindingType::Uniform,
                        has_dynamic_offset: false,
                        min_binding_size: None,
                    },
                    count: None,
                },
                // u_alpha_blend: f32
                wgpu::BindGroupLayoutEntry {
                    binding: 7,
                    visibility: ShaderStages::FRAGMENT,
                    ty: wgpu::BindingType::Buffer {
                        ty: wgpu::BufferBindingType::Uniform,
                        has_dynamic_offset: false,
                        min_binding_size: None,
                    },
                    count: None,
                },
                // u_alpha_cutoff: f32
                wgpu::BindGroupLayoutEntry {
                    binding: 8,
                    visibility: ShaderStages::FRAGMENT,
                    ty: wgpu::BindingType::Buffer {
                        ty: wgpu::BufferBindingType::Uniform,
                        has_dynamic_offset: false,
                        min_binding_size: None,
                    },
                    count: None,
                },
            ],
        }
    }
}
