use std::path::Path;

use serde::{Deserialize, Serialize};

use crate::{scene::ImportData, shader::ShaderFlags, Result, Vec3, Vec4};

use super::store_texture::{StoreTexture, TextureType};
/// The alpha rendering mode of a material.
#[derive(Clone, Copy, Eq, PartialEq, Debug, Serialize, Deserialize)]
pub enum AlphaMode {
    /// The alpha value is ignored and the rendered output is fully opaque.
    Opaque = 1,

    /// The rendered output is either fully opaque or fully transparent depending on
    /// the alpha value and the specified alpha cutoff value.
    Mask,

    /// The alpha value is used, to determine the transparency of the rendered output.
    /// The alpha cutoff value is ignored.
    Blend,
}

impl Into<AlphaMode> for gltf::material::AlphaMode {
    fn into(self) -> AlphaMode {
        match self {
            Self::Opaque => AlphaMode::Opaque,
            Self::Mask => AlphaMode::Mask,
            Self::Blend => AlphaMode::Blend,
        }
    }
}

impl Into<gltf::material::AlphaMode> for AlphaMode {
    fn into(self) -> gltf::material::AlphaMode {
        use gltf::material::AlphaMode::*;
        match self {
            Self::Opaque => Opaque,
            Self::Mask => Mask,
            Self::Blend => Blend,
        }
    }
}

#[derive(Debug, Serialize, Deserialize)]
pub struct StoreMaterial {
    pub index: usize,
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
    pub alpha_mode: AlphaMode,

    pub double_sided: bool,
    pub shader_flags: ShaderFlags,
    // pub pbr_shader: StoreShader,
}

impl StoreMaterial {
    pub fn from_gltf(
        g_material: &gltf::material::Material<'_>,
        textures: &mut Vec<StoreTexture>,
        imp: &ImportData,
        shader_flags: ShaderFlags,
        base_path: &Path,
        object_name: &String,
        material_name: &String,
    ) -> Result<Self> {
        let pbr = g_material.pbr_metallic_roughness();
        let mut base_color_texture = None;
        if let Some(color_info) = pbr.base_color_texture() {
            base_color_texture = load_store_texture(
                &color_info.texture(),
                textures,
                imp,
                base_path,
                TextureType::base_color,
                object_name,
                material_name,
            )?;
        }
        let mut metallic_roughness_texture = None;
        if let Some(mr_info) = pbr.metallic_roughness_texture() {
            metallic_roughness_texture = load_store_texture(
                &mr_info.texture(),
                textures,
                imp,
                base_path,
                TextureType::metallic_roughness,
                object_name,
                material_name,
            )?;
        }
        let mut normal_texture = None;
        let mut normal_scale = None;
        if let Some(norm_tex) = g_material.normal_texture() {
            normal_texture = load_store_texture(
                &norm_tex.texture(),
                textures,
                imp,
                base_path,
                TextureType::normal,
                object_name,
                material_name,
            )?;
            normal_scale = Some(norm_tex.scale());
        }
        let mut occlusion_texture = None;
        let mut occlusion_strength = 0.0;
        if let Some(occ_tex) = g_material.occlusion_texture() {
            occlusion_texture = load_store_texture(
                &occ_tex.texture(),
                textures,
                imp,
                base_path,
                TextureType::occlusion,
                object_name,
                material_name,
            )?;
            occlusion_strength = occ_tex.strength();
        }
        let mut emissive_texture = None;
        if let Some(em_info) = g_material.emissive_texture() {
            emissive_texture = load_store_texture(
                &em_info.texture(),
                textures,
                imp,
                base_path,
                TextureType::emissive,
                object_name,
                material_name,
            )?;
        }

        Ok(Self {
            index: g_material.index().unwrap_or(0),
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
            alpha_mode: g_material.alpha_mode().into(),
            double_sided: g_material.double_sided(),
            shader_flags,
        })
    }
}

fn load_store_texture(
    g_texture: &gltf::texture::Texture<'_>,
    textures: &mut Vec<StoreTexture>,
    imp: &ImportData,
    base_path: &Path,
    mat_ty: TextureType,
    object_name: &String,
    material_name: &String,
) -> Result<Option<usize>> {
    if let Some(tex) = textures.iter().find(|tex| tex.index == g_texture.index()) {
        return Ok(Some(tex.index));
    }

    let texture = StoreTexture::from_gltf(
        g_texture,
        imp,
        base_path,
        mat_ty,
        object_name,
        material_name,
    )?;
    let tex = texture.index;
    textures.push(texture);
    Ok(Some(tex))
}
