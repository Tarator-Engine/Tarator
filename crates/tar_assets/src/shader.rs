use wgsl_preprocessor::WGSLType;

use crate::{WgpuInfo, primitive::{Vertex, Instance}, Result, uniform::Uniform};

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

pub struct MaterialInput {
    pub base_color_factor: [f32; 4],
    pub metallic_factor: f32,
    pub roughness_factor: f32,
    pub normal_scale: f32,
    pub occlusion_strength: f32,
    pub emissive_factor: [f32; 3],
    pub alpha_cutoff: f32,
}

impl WGSLType for MaterialInput {
    fn type_name() -> String {
        "MaterialInput".into()
    }

    fn string_definition(&self) -> String {
        format!(
            "
            {}(
                vec4<f32>({:?}),
                {},
                {},
                {},
                {},
                vec3<f32>({:?}),
                {},
            )
            ",
            Self::type_name(),
            self.base_color_factor,
            self.metallic_factor,
            self.roughness_factor,
            self.normal_scale,
            self.occlusion_strength,
            self.emissive_factor,
            self.alpha_cutoff,
        )
    }
}

pub struct Shader {
    pub module: wgpu::ShaderModule,
}
impl Shader {
    pub fn from_path(path: &str, layouts: &[(wgpu::BindGroupLayoutDescriptor, Vec<(String, String)>)], defines: &[String], mat_in: MaterialInput, w_info: &WgpuInfo) -> Result<Self> {

        let mut binding = wgsl_preprocessor::ShaderBuilder::new(path).map_err(|e| crate::Error::NoFileExtension)?;
            
        let shader = binding.bind_groups_from_layouts(layouts)
            .parse_defines(defines)
            .put_constant("material", mat_in)
            .build();

        let module = w_info.device.create_shader_module(shader);

        Ok(Self {
            module
        })
    }
}


pub struct PbrShader {
    pub shader: Shader,
    pub flags: ShaderFlags,
    pub uniforms: PerFrameUniforms,
    pub pipeline: wgpu::RenderPipeline,
}

impl PbrShader {
    pub fn new(flags: ShaderFlags, mat_in: MaterialInput, w_info: &WgpuInfo) -> Result<Self> {
        let per_frame = (PerFrameUniforms::bind_group_layout(), PerFrameUniforms::names());
        let per_frame_bind_group = w_info.device.create_bind_group_layout(&per_frame.0);
        let shader = Shader::from_path(
            "shaders/static_pbr.wgsl",
            &[per_frame],
            &flags.as_strings(),
            mat_in,
            w_info)?;
        
        let uniforms = PerFrameUniforms::new(PerFrameData::new(), &per_frame_bind_group, w_info);

        let pipeline_layout = w_info.device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
            label: Some("Shader Render Pipline Layout"),
            bind_group_layouts: &[
                &per_frame_bind_group
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
            shader,
            flags,
            uniforms,
            pipeline,
        })
    }
}

pub trait BindGroup {
    type Data;
    fn new(data: Self::Data, layout: &wgpu::BindGroupLayout, w_info: &WgpuInfo) -> Self;
    fn bind_group_layout() -> wgpu::BindGroupLayoutDescriptor<'static>;
    fn names() -> Vec<(String, String)>;
}

pub struct PerStaticMaterialData {
    
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

    bind_group: Option<wgpu::BindGroup>,
}
pub struct PerFrameData {
    u_mpv_matrix: [[f32; 4]; 4],
    u_model_matrix: [[f32; 4]; 4],
    u_camera: [f32; 3],

    u_light_direction: [f32; 3],
    u_light_color: [f32; 3],

    u_ambient_light_color: [f32; 3],
    u_ambient_light_intensity: f32,

    u_alpha_blend: f32,
    u_alpha_cutoff: f32,
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
            u_alpha_cutoff: 0.0 
        }
    }
}

impl BindGroup for PerFrameUniforms {
    type Data = PerFrameData;
    fn new(data: PerFrameData, layout: &wgpu::BindGroupLayout, w_info: &WgpuInfo) -> Self {
        let mut uni = PerFrameUniforms { 
            u_mpv_matrix: Uniform::new(data.u_mpv_matrix, "u_mpv_matrix".into(), w_info), 
            u_model_matrix: Uniform::new(data.u_model_matrix, "u_model_matrix".into(), w_info), 
            u_camera: Uniform::new(data.u_camera, "u_camera".into(), w_info), 
            u_light_direction: Uniform::new(data.u_light_direction, "u_light_direction".into(), w_info), 
            u_light_color: Uniform::new(data.u_light_color, "u_light_color".into(), w_info), 
            u_ambient_light_color: Uniform::new(data.u_ambient_light_color, "u_ambient_light_color".into(), w_info), 
            u_ambient_light_intensity: Uniform::new(data.u_ambient_light_intensity, "u_ambient_light_intensity".into(), w_info), 
            u_alpha_blend: Uniform::new(data.u_alpha_blend, "u_alpha_blend".into(), w_info), 
            u_alpha_cutoff: Uniform::new(data.u_alpha_cutoff, "u_alpha_cutoff".into(), w_info),
            bind_group: None,
        };

        uni.bind_group = Some(w_info.device.create_bind_group(&wgpu::BindGroupDescriptor {
            label: Some("per frame bind group"),
            layout,
            entries: &[
                wgpu::BindGroupEntry {
                    binding: 0,
                    resource: uni.u_mpv_matrix.buff.as_entire_binding(),
                },
                wgpu::BindGroupEntry {
                    binding: 1,
                    resource: uni.u_model_matrix.buff.as_entire_binding(),
                },
                wgpu::BindGroupEntry {
                    binding: 2,
                    resource: uni.u_camera.buff.as_entire_binding(),
                },
                wgpu::BindGroupEntry {
                    binding: 3,
                    resource: uni.u_light_direction.buff.as_entire_binding(),
                },
                wgpu::BindGroupEntry {
                    binding: 4,
                    resource: uni.u_light_color.buff.as_entire_binding(),
                },
                wgpu::BindGroupEntry {
                    binding: 5,
                    resource: uni.u_ambient_light_color.buff.as_entire_binding(),
                },
                wgpu::BindGroupEntry {
                    binding: 6,
                    resource: uni.u_ambient_light_intensity.buff.as_entire_binding(),
                },
                wgpu::BindGroupEntry {
                    binding: 7,
                    resource: uni.u_alpha_blend.buff.as_entire_binding(),
                },
                wgpu::BindGroupEntry {
                    binding: 8,
                    resource: uni.u_alpha_cutoff.buff.as_entire_binding(),
                },
            ]
        }));
        
        uni
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
                    count: None
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
                    count: None
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
                    count: None
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
                    count: None
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
                    count: None
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
                    count: None
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
                    count: None
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
                    count: None
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
                    count: None
                },
            ]
        }
    }
}

// /// NOTE: this has to match the shader
// pub struct PbrUniforms {
//     pub bind_group: wgpu::BindGroup,
//     pub u_MPVMatrix: Uniform<[[f32; 4]; 4]>,
//     pub u_ModelMatrix: Uniform<[[f32; 4]; 4]>,
//     pub u_Camera: Uniform<[f32; 3]>,

//     pub u_LightDirection: Uniform<[f32; 3]>,
//     pub u_LightColor: Uniform<[f32; 3]>,
    
//     pub u_AmbientLightColor: Uniform<[f32; 3]>,
//     pub u_AmbientLightIntensity: Uniform<f32>,

//     pub u_AlphaBlend: Uniform<f32>,
//     pub u_AlphaCutoff: Uniform<f32>,

//     pub u_ShaderFlags: Uniform<u32>,
// }

// /// NOTE: this has to match the [`PbrUniforms`]
// pub struct PbrUniformData {
//     pub u_MPVMatrix: [[f32; 4]; 4],
//     pub u_ModelMatrix: [[f32; 4]; 4],
//     pub u_Camera: [f32; 3],

//     pub u_LightDirection: [f32; 3],
//     pub u_LightColor: [f32; 3],
    
//     pub u_AmbientLightColor: [f32; 3],
//     pub u_AmbientLightIntensity: f32,

//     pub u_AlphaBlend: f32,
//     pub u_AlphaCutoff: f32,

//     pub u_ShaderFlags: u32,
// }

// impl PbrUniforms {
//     pub fn new(info: PbrUniformData, layout: &wgpu::BindGroupLayout, w_info: &WgpuInfo) -> Self {
//         let u_MPVMatrix = Uniform::new(info.u_MPVMatrix, "u_MPVMatrix".into(), w_info);
//         let u_ModelMatrix = Uniform::new(info.u_ModelMatrix, "u_ModelMatrix".into(), w_info);
//         let u_Camera = Uniform::new(info.u_Camera, "u_Camera".into(), w_info);
        
//         let u_LightDirection = Uniform::new(info.u_LightDirection, "u_LightDirection".into(), w_info);
//         let u_LightColor = Uniform::new(info.u_LightColor, "u_LightColor".into(), w_info);
        
//         let u_AmbientLightColor = Uniform::new(info.u_AmbientLightColor, "u_AmbientLightColor".into(), w_info);
//         let u_AmbientLightIntensity = Uniform::new(info.u_AmbientLightIntensity, "u_AmbientLightIntensity".into(), w_info);
        
//         let u_AlphaBlend = Uniform::new(info.u_AlphaBlend, "u_AlphaBlend".into(), w_info);
//         let u_AlphaCutoff = Uniform::new(info.u_AlphaCutoff, "u_AlphaCutoff".into(), w_info);

//         let u_ShaderFlags = Uniform::new(info.u_ShaderFlags, "u_ShaderFlags".into(), w_info);

//         // NOTE: this has to match [`PbrUniforms`]
//         let bind_group = w_info.device.create_bind_group(&wgpu::BindGroupDescriptor {
//             layout,
//             entries: &[
//                 wgpu::BindGroupEntry {
//                     binding: 0,
//                     resource: u_MPVMatrix.buff.as_entire_binding(),
//                 },
//                 wgpu::BindGroupEntry {
//                     binding: 1,
//                     resource: u_ModelMatrix.buff.as_entire_binding(),
//                 },
//                 wgpu::BindGroupEntry {
//                     binding: 2,
//                     resource: u_Camera.buff.as_entire_binding(),
//                 },
//                 wgpu::BindGroupEntry {
//                     binding: 3,
//                     resource: u_LightDirection.buff.as_entire_binding(),
//                 },
//                 wgpu::BindGroupEntry {
//                     binding: 4,
//                     resource: u_LightColor.buff.as_entire_binding(),
//                 },
//                 wgpu::BindGroupEntry {
//                     binding: 5,
//                     resource: u_AmbientLightColor.buff.as_entire_binding(),
//                 },
//                 wgpu::BindGroupEntry {
//                     binding: 6,
//                     resource: u_AmbientLightIntensity.buff.as_entire_binding(),
//                 },
//                 wgpu::BindGroupEntry {
//                     binding: 7,
//                     resource: u_AlphaBlend.buff.as_entire_binding(),
//                 },
//                 wgpu::BindGroupEntry {
//                     binding: 8,
//                     resource: u_AlphaCutoff.buff.as_entire_binding(),
//                 },
//                 wgpu::BindGroupEntry {
//                     binding: 9,
//                     resource: u_ShaderFlags.buff.as_entire_binding(),
//                 }
//             ],
//             label: Some("shader uniforms bind_group"),
//         });

//         Self {
//             bind_group,
//             u_MPVMatrix,
//             u_ModelMatrix,
//             u_Camera,

//             u_LightDirection,
//             u_LightColor,

//             u_AmbientLightColor,
//             u_AmbientLightIntensity,

//             u_AlphaBlend,
//             u_AlphaCutoff,

//             u_ShaderFlags,
//         }
//     }

//     /// NOTE: this has to match [`PbrUniforms`]
//     pub fn gen_bind_group(w_info: &WgpuInfo) -> wgpu::BindGroupLayout {
//         w_info.device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
//             label: Some("shader uniforms bind_group_layout"),
//             entries: &[
//                 // u_MPVMatrix
//                 wgpu::BindGroupLayoutEntry {
//                     binding: 0,
//                     visibility: wgpu::ShaderStages::VERTEX,
//                     ty: wgpu::BindingType::Buffer { 
//                         ty: wgpu::BufferBindingType::Uniform,
//                         has_dynamic_offset: false,
//                         min_binding_size: None,
//                     },
//                     count: None,
//                 },
//                 // u_ModelMatrix
//                 wgpu::BindGroupLayoutEntry {
//                     binding: 1,
//                     visibility: wgpu::ShaderStages::VERTEX,
//                     ty: wgpu::BindingType::Buffer { 
//                         ty: wgpu::BufferBindingType::Uniform,
//                         has_dynamic_offset: false,
//                         min_binding_size: None,
//                     },
//                     count: None,
//                 },
//                 // u_Camera
//                 wgpu::BindGroupLayoutEntry {
//                     binding: 2,
//                     visibility: wgpu::ShaderStages::FRAGMENT,
//                     ty: wgpu::BindingType::Buffer { 
//                         ty: wgpu::BufferBindingType::Uniform,
//                         has_dynamic_offset: false,
//                         min_binding_size: None,
//                     },
//                     count: None,
//                 },
//                 // u_LightDirection
//                 wgpu::BindGroupLayoutEntry {
//                     binding: 3,
//                     visibility: wgpu::ShaderStages::FRAGMENT,
//                     ty: wgpu::BindingType::Buffer { 
//                         ty: wgpu::BufferBindingType::Uniform,
//                         has_dynamic_offset: false,
//                         min_binding_size: None,
//                     },
//                     count: None,
//                 },
//                 // u_LightColor
//                 wgpu::BindGroupLayoutEntry {
//                     binding: 4,
//                     visibility: wgpu::ShaderStages::FRAGMENT,
//                     ty: wgpu::BindingType::Buffer { 
//                         ty: wgpu::BufferBindingType::Uniform,
//                         has_dynamic_offset: false,
//                         min_binding_size: None,
//                     },
//                     count: None,
//                 },
//                 // u_AmbientLightColor
//                 wgpu::BindGroupLayoutEntry {
//                     binding: 5,
//                     visibility: wgpu::ShaderStages::FRAGMENT,
//                     ty: wgpu::BindingType::Buffer { 
//                         ty: wgpu::BufferBindingType::Uniform,
//                         has_dynamic_offset: false,
//                         min_binding_size: None,
//                     },
//                     count: None,
//                 },
//                 // u_AmbientLightIntensity
//                 wgpu::BindGroupLayoutEntry {
//                     binding: 6,
//                     visibility: wgpu::ShaderStages::FRAGMENT,
//                     ty: wgpu::BindingType::Buffer { 
//                         ty: wgpu::BufferBindingType::Uniform,
//                         has_dynamic_offset: false,
//                         min_binding_size: None,
//                     },
//                     count: None,
//                 },
//                 // u_AlphaBlend
//                 wgpu::BindGroupLayoutEntry {
//                     binding: 7,
//                     visibility: wgpu::ShaderStages::FRAGMENT,
//                     ty: wgpu::BindingType::Buffer { 
//                         ty: wgpu::BufferBindingType::Uniform,
//                         has_dynamic_offset: false,
//                         min_binding_size: None,
//                     },
//                     count: None,
//                 },
//                 // u_AlphaCutoff
//                 wgpu::BindGroupLayoutEntry {
//                     binding: 8,
//                     visibility: wgpu::ShaderStages::FRAGMENT,
//                     ty: wgpu::BindingType::Buffer { 
//                         ty: wgpu::BufferBindingType::Uniform,
//                         has_dynamic_offset: false,
//                         min_binding_size: None,
//                     },
//                     count: None,
//                 },
//                 // u_ShaderFlags
//                 wgpu::BindGroupLayoutEntry {
//                     binding: 9,
//                     visibility: wgpu::ShaderStages::VERTEX_FRAGMENT,
//                     ty: wgpu::BindingType::Buffer { 
//                         ty: wgpu::BufferBindingType::Uniform,
//                         has_dynamic_offset: false,
//                         min_binding_size: None,
//                     },
//                     count: None,
//                 },
//             ]
//         })
//     }
// }