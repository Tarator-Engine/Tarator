use std::path::Path;

use crate::{scene::ImportData, shader::ShaderFlags, Result, Vec3, Vec4};

use super::store_texture::StoreTexture;

pub struct StoreMaterial {
    pub index: Option<usize>,
    pub name: Option<String>,

    pub base_color_factor: Vec4,
    pub base_color_texture: Option<usize>,
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
        textures: &mut Vec<StoreTexture>,
        imp: &ImportData,
        shader_flags: ShaderFlags,
        base_path: &Path,
    ) -> Result<Self> {
        let pbr = g_material.pbr_metallic_roughness();
        let mut base_color_texture = None;
        if let Some(color_info) = pbr.base_color_texture() {
            base_color_texture = load_store_texture(
                &color_info.texture(),
                color_info.tex_coord(),
                textures,
                imp,
                base_path,
            )?;
        }
        let mut metallic_roughness_texture = None;
        if let Some(mr_info) = pbr.metallic_roughness_texture() {
            metallic_roughness_texture = load_store_texture(
                &mr_info.texture(),
                mr_info.tex_coord(),
                textures,
                imp,
                base_path,
            )?;
        }
        let mut normal_texture = None;
        let mut normal_scale = None;
        if let Some(norm_tex) = g_material.normal_texture() {
            normal_texture = load_store_texture(
                &norm_tex.texture(),
                norm_tex.tex_coord(),
                textures,
                imp,
                base_path,
            )?;
            normal_scale = Some(norm_tex.scale());
        }
        let mut occlusion_texture = None;
        let mut occlusion_strength = 0.0;
        if let Some(occ_tex) = g_material.occlusion_texture() {
            occlusion_texture = load_store_texture(
                &occ_tex.texture(),
                occ_tex.tex_coord(),
                textures,
                imp,
                base_path,
            )?;
            occlusion_strength = occ_tex.strength();
        }
        let mut emissive_texture = None;
        if let Some(em_info) = g_material.emissive_texture() {
            emissive_texture = load_store_texture(
                &em_info.texture(),
                em_info.tex_coord(),
                textures,
                imp,
                base_path,
            )?;
        }

        Ok(Self {
            index: g_material.index(),
            name: g_material.name().map(|s| s.into()),
            base_color_factor: pbr.base_color_factor().into(),
            base_color_texture,
            metallic_factor: pbr.metallic_factor(),
            roughness_factor: pbr.roughness_factor(),
            metallic_roughness_texture,
            normal_texture,
            normal_scale,
            occlusion_texture,
            occlusion_strength,
            emissive_factor: g_material.emissive_factor().into(),
            emissive_texture,
            alpha_cutoff: g_material.alpha_cutoff(),
            alpha_mode: g_material.alpha_mode(),
            double_sided: g_material.double_sided(),
        })
    }
}

fn load_store_texture(
    g_texture: &gltf::texture::Texture<'_>,
    tex_coords: u32,
    textures: &mut Vec<StoreTexture>,
    imp: &ImportData,
    base_path: &Path,
) -> Result<Option<usize>> {
    if let Some(tex) = textures.iter().find(|tex| tex.index == g_texture.index()) {
        return Ok(Some(tex.index));
    }

    let texture = StoreTexture::from_gltf(g_texture, tex_coords, imp, base_path)?;
    let tex = texture.index;
    textures.push(texture);
    Ok(Some(tex))
}
