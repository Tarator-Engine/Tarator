use cgmath::{Vector3, Vector4};

use crate::{
    primitive::Instance,
    shader::{MaterialInput, PbrShader, ShaderFlags},
    texture::Texture,
    uniform::Uniform,
    vertex::Vertex,
    Result, Vec3Slice, Vec4Slice, WgpuInfo,
};

pub struct PbrMaterial {
    pub index: Option<usize>,
    pub name: String,

    pub base_color_factor: Vector4<f32>,
    pub base_color_texture: Option<Texture>,
    pub metallic_factor: f32,
    pub roughness_factor: f32,
    pub metallic_roughness_texture: Option<Texture>,

    pub normal_texture: Option<Texture>,
    pub normal_scale: Option<f32>,

    pub occlusion_texture: Option<Texture>,
    pub occlusion_strength: f32,
    pub emissive_factor: Vector3<f32>,
    pub emissive_texture: Option<Texture>,

    pub alpha_cutoff: Option<f32>,
    pub alpha_mode: gltf::material::AlphaMode,

    pub double_sided: bool,

    pub pbr_shader: PbrShader,

    pub per_frame_uniforms: PerFrameUniforms,
    pub pipeline: wgpu::RenderPipeline,
}

impl PbrMaterial {
    pub fn new(
        index: Option<usize>,
        name: String,

        base_color_factor: Vector4<f32>,
        base_color_texture: Option<Texture>,
        metallic_factor: f32,
        roughness_factor: f32,
        metallic_roughness_texture: Option<Texture>,

        normal_texture: Option<Texture>,
        normal_scale: Option<f32>,

        occlusion_texture: Option<Texture>,
        occlusion_strength: f32,
        emissive_factor: Vector3<f32>,
        emissive_texture: Option<Texture>,

        alpha_cutoff: Option<f32>,
        alpha_mode: gltf::material::AlphaMode,

        double_sided: bool,

        shader_flags: ShaderFlags,
        w_info: &WgpuInfo,
    ) -> Result<Self> {
        let shader_flags = Self::shader_flags(
            base_color_texture.is_some(),
            normal_texture.is_some(),
            emissive_texture.is_some(),
            metallic_roughness_texture.is_some(),
            occlusion_texture.is_some(),
        ) | shader_flags;

        let pbr_shader = PbrShader::new(
            shader_flags,
            MaterialInput {
                base_color_factor: base_color_factor.as_slice(),
                metallic_factor: metallic_factor,
                roughness_factor: roughness_factor,
                normal_scale: normal_scale.unwrap_or(1.0),
                occlusion_strength: occlusion_strength,
                emissive_factor: emissive_factor.as_slice(),
                alpha_cutoff: alpha_cutoff.unwrap_or(1.0),
            },
            w_info,
        )?;

        let per_frame_bind_group = w_info
            .device
            .create_bind_group_layout(&PerFrameUniforms::bind_group_layout());

        let per_frame_uniforms =
            PerFrameUniforms::new(PerFrameData::new(), &per_frame_bind_group, w_info);

        let pipeline_layout =
            w_info
                .device
                .create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
                    label: Some("Material pipeline layout"),
                    bind_group_layouts: &[&per_frame_bind_group],
                    push_constant_ranges: &[],
                });

        let pipeline = w_info
            .device
            .create_render_pipeline(&wgpu::RenderPipelineDescriptor {
                label: Some(&format!("{:?}", pbr_shader.shader.module)),
                layout: Some(&pipeline_layout),
                vertex: wgpu::VertexState {
                    module: &pbr_shader.shader.module,
                    entry_point: "vs_main",
                    buffers: &[Vertex::desc(), Instance::desc()],
                },
                fragment: Some(wgpu::FragmentState {
                    module: &pbr_shader.shader.module,
                    entry_point: "fs_main",
                    targets: &[Some(wgpu::ColorTargetState {
                        format: w_info.surface_format,
                        blend: Some(wgpu::BlendState {
                            alpha: wgpu::BlendComponent::REPLACE,
                            color: wgpu::BlendComponent::REPLACE,
                        }),
                        write_mask: wgpu::ColorWrites::ALL,
                    })],
                }),
                primitive: wgpu::PrimitiveState {
                    topology: wgpu::PrimitiveTopology::TriangleList,
                    strip_index_format: None,
                    front_face: wgpu::FrontFace::Ccw,
                    cull_mode: Some(wgpu::Face::Back),
                    polygon_mode: wgpu::PolygonMode::Fill,
                    unclipped_depth: false,
                    conservative: false,
                },
                depth_stencil: Some(wgpu::DepthStencilState {
                    format: wgpu::TextureFormat::Depth32Float,
                    depth_compare: wgpu::CompareFunction::Less,
                    stencil: wgpu::StencilState::default(),
                    bias: wgpu::DepthBiasState::default(),
                    depth_write_enabled: true,
                }),
                multisample: wgpu::MultisampleState {
                    count: 1,
                    mask: !0,
                    alpha_to_coverage_enabled: false,
                },
                // If the pipeline will be used with a multiview render pass, this
                // indicates how many array layers the attachments will have.
                multiview: None,
            });

        Ok(Self {
            index,
            name,
            base_color_factor,
            base_color_texture,
            metallic_factor,
            roughness_factor,
            metallic_roughness_texture,
            normal_texture,
            normal_scale,
            occlusion_texture,
            occlusion_strength,
            emissive_factor,
            emissive_texture,
            alpha_cutoff,
            alpha_mode,
            double_sided,
            pbr_shader,
            per_frame_uniforms,
            pipeline,
        })
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

    pub fn set_render_pass<'a, 'b>(&'a self, render_pass: &'b mut wgpu::RenderPass<'a>) {
        render_pass.set_pipeline(&self.pipeline);
    }

    pub fn set_bind_groups<'a, 'b>(
        &'a self,
        render_pass: &'b mut wgpu::RenderPass<'a>,
        start: u32,
    ) {
        render_pass.set_bind_group(start, &self.per_frame_uniforms.bind_group, &[]);
    }
}

pub trait BindGroup {
    type Data;
    fn new(data: Self::Data, layout: &wgpu::BindGroupLayout, w_info: &WgpuInfo) -> Self;
    fn bind_group_layout() -> wgpu::BindGroupLayoutDescriptor<'static>;
    fn names() -> Vec<(String, String)>;
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

impl PerFrameData {
    pub fn new() -> Self {
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
    fn new(data: PerFrameData, layout: &wgpu::BindGroupLayout, w_info: &WgpuInfo) -> Self {
        let u_mpv_matrix = Uniform::new(data.u_mpv_matrix, "u_mpv_matrix".into(), w_info);
        let u_model_matrix = Uniform::new(data.u_model_matrix, "u_model_matrix".into(), w_info);
        let u_camera = Uniform::new(data.u_camera, "u_camera".into(), w_info);
        let u_light_direction =
            Uniform::new(data.u_light_direction, "u_light_direction".into(), w_info);
        let u_light_color = Uniform::new(data.u_light_color, "u_light_color".into(), w_info);
        let u_ambient_light_color = Uniform::new(
            data.u_ambient_light_color,
            "u_ambient_light_color".into(),
            w_info,
        );
        let u_ambient_light_intensity = Uniform::new(
            data.u_ambient_light_intensity,
            "u_ambient_light_intensity".into(),
            w_info,
        );
        let u_alpha_blend = Uniform::new(data.u_alpha_blend, "u_alpha_blend".into(), w_info);
        let u_alpha_cutoff = Uniform::new(data.u_alpha_cutoff, "u_alpha_cutoff".into(), w_info);

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

    fn bind_group_layout() -> wgpu::BindGroupLayoutDescriptor<'static> {
        use wgpu::ShaderStages;
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
