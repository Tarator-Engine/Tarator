use crate::{scene::ImportData, shader::ShaderFlags, Vec3, Vec4};

pub struct StoreMaterial {
    pub index: Option<usize>,
    pub name: Option<String>,

    pub base_color_factor: Vec4,
    pub base_color_texture: Option<String>,
    pub metallic_factor: f32,
    pub roughness_factor: f32,
    pub metallic_roughness_texture: Option<usize>,

    pub normal_texture: Option<usize>,
    pub normal_scale: Option<f32>,

    pub occlusion_texture: Option<usize>,
    pub occlusion_strength: f32,
    pub emissive_factor: Vec3,
    pub emissive_texture: Option<usize>,

    pub alpha_cutoff: Option<f32>,
    pub alpha_mode: gltf::material::AlphaMode,

    pub double_sided: bool,
    // pub pbr_shader: StoreShader,
}

impl StoreMaterial {
    pub fn from_gltf(
        g_material: &gltf::material::Material<'_>,
        imp: &ImportData,
        shader_flags: ShaderFlags,
    ) -> Self {

        let pbr = g_material.pbr_metallic_roughness();

        let base_color_texture = pbr.base_color_texture().map(|info| {
            
        })

        Self {
            index: g_material.index(),
            name: g_material.name(),
            base_color_factor: pbr.base_color_factor(),
            base_color_texture: pbr.base_color_texture()
        }

        todo!()
    }
}
