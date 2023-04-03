use std::sync::Arc;

use serde::{Deserialize, Serialize};
use wgpu::BindGroupLayoutDescriptor;
use wgsl_preprocessor::WGSLType;

use crate::{Result, WgpuInfo};

bitflags! {
    /// Flags matching the defines in the PBR shader
    #[derive(Serialize, Deserialize)]
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
    #[must_use] pub fn as_strings(self) -> Vec<String> {
        (0..15)
            .map(|i| 1u16 << i)
            .filter(|i| self.bits & i != 0)
            .map(|i| format!("{:?}", Self::from_bits_truncate(i)))
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
let material = {}(
    vec4<f32>({:?}),
    {},
    {},
    {},
    {},
    vec3<f32>({:?}),
    {}
);
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
        .replace(['[', ']'], "")
    }
}

pub struct Shader {
    pub module: wgpu::ShaderModule,
}
impl Shader {
    pub fn from_path(
        path: &str,
        layouts: &[(wgpu::BindGroupLayoutDescriptor, Vec<(String, String)>)],
        defines: &[String],
        mat_in: MaterialInput,
        w_info: Arc<WgpuInfo>,
    ) -> Result<Self> {
        println!("importing shader {path}");
        let mut binding = wgsl_preprocessor::ShaderBuilder::new(path, Some(defines))?;

        let shader = binding
            .bind_groups_from_layouts(layouts)
            .put_constant("material_base_color_factor", mat_in.base_color_factor)
            .put_constant("material_metallic_factor", mat_in.metallic_factor)
            .put_constant("material_roughness_factor", mat_in.roughness_factor)
            .put_constant("material_normal_scale", mat_in.normal_scale)
            .put_constant("material_occlusion_strength", mat_in.occlusion_strength)
            .put_constant("material_emissive_factor", mat_in.emissive_factor)
            .put_constant("material_alpha_cutoff", mat_in.alpha_cutoff);

        let shader = shader.build();

        let module = w_info.device.create_shader_module(shader);

        Ok(Self { module })
    }
}

pub struct PbrShader {
    pub shader: Shader,
    pub flags: ShaderFlags,
}

impl PbrShader {
    pub fn new(
        flags: ShaderFlags,
        mat_in: MaterialInput,
        layouts: &[(BindGroupLayoutDescriptor, Vec<(String, String)>)],
        w_info: Arc<WgpuInfo>,
    ) -> Result<Self> {
        // let per_frame_bind_group = w_info.device.create_bind_group_layout(&per_frame.0);
        let shader = Shader::from_path(
            "shaders/static_pbr.wgsl",
            layouts,
            &flags.as_strings(),
            mat_in,
            w_info,
        )?;

        Ok(Self { shader, flags })
    }
}
