use crate::{WgpuInfo, uniform::{Uniform, self}, primitive::{Vertex, Instance}};

bitflags! {
    /// Flags matching the defines in the PBR shader
    pub struct ShaderFlags: u16 {
        // vertex shader + fragment shader
        const HAS_NORMALS           = 1;
        const HAS_TANGENTS          = 1 << 1;
        const HAS_UV                = 1 << 2;
        const HAS_COLORS            = 1 << 3;

        // fragment shader only
        const USE_IBL               = 1 << 4;
        const HAS_BASECOLORMAP      = 1 << 5;
        const HAS_NORMALMAP         = 1 << 6;
        const HAS_EMISSIVEMAP       = 1 << 7;
        const HAS_METALROUGHNESSMAP = 1 << 8;
        const HAS_OCCLUSIONMAP      = 1 << 9;
        const USE_TEX_LOD           = 1 << 10;
    }
}

impl ShaderFlags {
    pub fn as_strings(self) -> Vec<String> {
        (0..15)
            .map(|i| 1u16 << i)
            .filter(|i| self.bits & i != 0)
            .map(|i| format!("{:?}", ShaderFlags::from_bits_truncate(i)))
            .collect()
    }   
}

pub struct Shader {
    pub module: wgpu::ShaderModule,
}
impl Shader {
    pub fn from_source(shader_code: &str, defines: &[String], w_info: &WgpuInfo) -> Self {
        let shader = wgpu::ShaderModuleDescriptor {
            label: Some("pbr shader"),
            source: wgpu::ShaderSource::Wgsl(shader_code.into()),
        };

        let module = w_info.device.create_shader_module(shader);

        Self {
            module
        }
    }
}


pub struct PbrShader {
    pub shader: Shader,
    pub flags: ShaderFlags,
    pub uniforms: PbrUniforms,
    pub pipeline: wgpu::RenderPipeline,
    pub bind_group_layout: wgpu::BindGroupLayout
}

impl PbrShader {
    pub fn new(flags: ShaderFlags, info: PbrUniformData, camera_layout: &wgpu::BindGroupLayout, color_format: wgpu::TextureFormat, w_info: &WgpuInfo) -> Self {
        let mut shader = Shader::from_source(
            include_str!("shaders/pbr.wgsl"),
            &flags.as_strings(),
            w_info);

        let bind_group_layout = PbrUniforms::gen_bind_group(w_info);

        let uniforms = PbrUniforms::new(info, &bind_group_layout, w_info);

        let pipeline_layout = w_info.device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
            label: Some("Shader Render Pipline Layout"),
            bind_group_layouts: &[
                &bind_group_layout,
                camera_layout,
            ],
            push_constant_ranges: &[]
        });
        let pipeline = 
            w_info.device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
                label: Some(&format!("{:?}", shader.module)),
                layout: Some(&pipeline_layout),
                vertex: wgpu::VertexState {
                    module: &shader.module,
                    entry_point: "vs_main",
                    buffers: &[Vertex::desc(), Instance::desc()]
                },
                fragment: Some(wgpu::FragmentState {
                    module: &shader.module,
                    entry_point: "fs_main",
                    targets: &[Some(wgpu::ColorTargetState {
                        format: color_format,
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

        Self {
            shader,
            flags,
            uniforms,
            bind_group_layout,
            pipeline,
        }
    }
}

pub struct PbrUniforms {
    pub bind_group: wgpu::BindGroup,
    pub u_MPVMatrix: Uniform<[[f32; 4]; 4]>,
    pub u_ModelMatrix: Uniform<[[f32; 4]; 4]>,
    pub u_Camera: Uniform<[f32; 3]>,

    pub u_LightDirection: Uniform<[f32; 3]>,
    pub u_LightColor: Uniform<[f32; 3]>,
    
    pub u_AmbientLightColor: Uniform<[f32; 3]>,
    pub u_AmbientLightIntensity: Uniform<f32>,

    pub u_AlphaBlend: Uniform<f32>,
    pub u_AlphaCutoff: Uniform<f32>,
}

pub struct PbrUniformData {
    pub u_MPVMatrix: [[f32; 4]; 4],
    pub u_ModelMatrix: [[f32; 4]; 4],
    pub u_Camera: [f32; 3],

    pub u_LightDirection: [f32; 3],
    pub u_LightColor: [f32; 3],
    
    pub u_AmbientLightColor: [f32; 3],
    pub u_AmbientLightIntensity: f32,

    pub u_AlphaBlend: f32,
    pub u_AlphaCutoff: f32,
}

impl PbrUniforms {
    pub fn new(info: PbrUniformData, layout: &wgpu::BindGroupLayout, w_info: &WgpuInfo) -> Self {
        let u_MPVMatrix = Uniform::new(info.u_MPVMatrix, "u_MPVMatrix".into(), w_info);
        let u_ModelMatrix = Uniform::new(info.u_ModelMatrix, "u_ModelMatrix".into(), w_info);
        let u_Camera = Uniform::new(info.u_Camera, "u_Camera".into(), w_info);
        
        let u_LightDirection = Uniform::new(info.u_LightDirection, "u_LightDirection".into(), w_info);
        let u_LightColor = Uniform::new(info.u_LightColor, "u_LightColor".into(), w_info);
        
        let u_AmbientLightColor = Uniform::new(info.u_AmbientLightColor, "u_AmbientLightColor".into(), w_info);
        let u_AmbientLightIntensity = Uniform::new(info.u_AmbientLightIntensity, "u_AmbientLightIntensity".into(), w_info);
        
        let u_AlphaBlend = Uniform::new(info.u_AlphaBlend, "u_AlphaBlend".into(), w_info);
        let u_AlphaCutoff = Uniform::new(info.u_AlphaCutoff, "u_AlphaCutoff".into(), w_info);

        let bind_group = w_info.device.create_bind_group(&wgpu::BindGroupDescriptor {
            layout,
            entries: &[
                wgpu::BindGroupEntry {
                    binding: 0,
                    resource: u_MPVMatrix.buff.as_entire_binding(),
                },
                wgpu::BindGroupEntry {
                    binding: 1,
                    resource: u_ModelMatrix.buff.as_entire_binding(),
                },
                wgpu::BindGroupEntry {
                    binding: 2,
                    resource: u_Camera.buff.as_entire_binding(),
                },
                wgpu::BindGroupEntry {
                    binding: 3,
                    resource: u_LightDirection.buff.as_entire_binding(),
                },
                wgpu::BindGroupEntry {
                    binding: 4,
                    resource: u_LightColor.buff.as_entire_binding(),
                },
                wgpu::BindGroupEntry {
                    binding: 5,
                    resource: u_AmbientLightColor.buff.as_entire_binding(),
                },
                wgpu::BindGroupEntry {
                    binding: 6,
                    resource: u_AmbientLightIntensity.buff.as_entire_binding(),
                },
                wgpu::BindGroupEntry {
                    binding: 7,
                    resource: u_AlphaBlend.buff.as_entire_binding(),
                },
                wgpu::BindGroupEntry {
                    binding: 8,
                    resource: u_AlphaCutoff.buff.as_entire_binding(),
                },
            ],
            label: Some("shader uniforms bind_group"),
        });

        Self {
            bind_group,
            u_MPVMatrix,
            u_ModelMatrix,
            u_Camera,

            u_LightDirection,
            u_LightColor,

            u_AmbientLightColor,
            u_AmbientLightIntensity,

            u_AlphaBlend,
            u_AlphaCutoff,
        }
    }

    pub fn gen_bind_group(w_info: &WgpuInfo) -> wgpu::BindGroupLayout {
        w_info.device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
            label: Some("shader uniforms bind_group_layout"),
            entries: &[
                // u_MPVMatrix
                wgpu::BindGroupLayoutEntry {
                    binding: 0,
                    visibility: wgpu::ShaderStages::VERTEX,
                    ty: wgpu::BindingType::Buffer { 
                        ty: wgpu::BufferBindingType::Uniform,
                        has_dynamic_offset: false,
                        min_binding_size: None,
                    },
                    count: None,
                },
                // u_ModelMatrix
                wgpu::BindGroupLayoutEntry {
                    binding: 1,
                    visibility: wgpu::ShaderStages::VERTEX,
                    ty: wgpu::BindingType::Buffer { 
                        ty: wgpu::BufferBindingType::Uniform,
                        has_dynamic_offset: false,
                        min_binding_size: None,
                    },
                    count: None,
                },
                // u_Camera
                wgpu::BindGroupLayoutEntry {
                    binding: 2,
                    visibility: wgpu::ShaderStages::FRAGMENT,
                    ty: wgpu::BindingType::Buffer { 
                        ty: wgpu::BufferBindingType::Uniform,
                        has_dynamic_offset: false,
                        min_binding_size: None,
                    },
                    count: None,
                },
                // u_LightDirection
                wgpu::BindGroupLayoutEntry {
                    binding: 1,
                    visibility: wgpu::ShaderStages::FRAGMENT,
                    ty: wgpu::BindingType::Buffer { 
                        ty: wgpu::BufferBindingType::Uniform,
                        has_dynamic_offset: false,
                        min_binding_size: None,
                    },
                    count: None,
                },
                // u_LightColor
                wgpu::BindGroupLayoutEntry {
                    binding: 1,
                    visibility: wgpu::ShaderStages::FRAGMENT,
                    ty: wgpu::BindingType::Buffer { 
                        ty: wgpu::BufferBindingType::Uniform,
                        has_dynamic_offset: false,
                        min_binding_size: None,
                    },
                    count: None,
                },
                // u_AmbientLightColor
                wgpu::BindGroupLayoutEntry {
                    binding: 1,
                    visibility: wgpu::ShaderStages::FRAGMENT,
                    ty: wgpu::BindingType::Buffer { 
                        ty: wgpu::BufferBindingType::Uniform,
                        has_dynamic_offset: false,
                        min_binding_size: None,
                    },
                    count: None,
                },
                // u_AmbientLightIntensity
                wgpu::BindGroupLayoutEntry {
                    binding: 1,
                    visibility: wgpu::ShaderStages::FRAGMENT,
                    ty: wgpu::BindingType::Buffer { 
                        ty: wgpu::BufferBindingType::Uniform,
                        has_dynamic_offset: false,
                        min_binding_size: None,
                    },
                    count: None,
                },
                // u_AlphaBlend
                wgpu::BindGroupLayoutEntry {
                    binding: 1,
                    visibility: wgpu::ShaderStages::FRAGMENT,
                    ty: wgpu::BindingType::Buffer { 
                        ty: wgpu::BufferBindingType::Uniform,
                        has_dynamic_offset: false,
                        min_binding_size: None,
                    },
                    count: None,
                },
                // u_AlphaCutoff
                wgpu::BindGroupLayoutEntry {
                    binding: 1,
                    visibility: wgpu::ShaderStages::FRAGMENT,
                    ty: wgpu::BindingType::Buffer { 
                        ty: wgpu::BufferBindingType::Uniform,
                        has_dynamic_offset: false,
                        min_binding_size: None,
                    },
                    count: None,
                },
            ]
        })
    }
}