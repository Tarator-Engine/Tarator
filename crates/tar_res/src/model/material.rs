use std::sync::Arc;

use scr_types::prims::{Vec3, Vec4};
use serde::{Deserialize, Serialize};

use super::serde_helpers::*;

#[derive(Debug, Serialize, Deserialize)]
pub struct Material {
    pub pbr: PbrMaterial,
    pub normal: Option<NormalMap>,
    pub occlusion: Option<Occlusion>,
    pub emissive: Emissive,
}

impl Material {
    pub fn new_from_gltf(material: Arc<easy_gltf::Material>) -> Self {
        let pbr = PbrMaterial::new_from_gltf(material.pbr.clone());
        let normal = material.normal.clone().map(NormalMap::new_from_gltf);
        let occlusion = material.occlusion.clone().map(Occlusion::new_from_gltf);
        let emissive = Emissive::new_from_gltf(material.emissive.clone());

        Self {
            pbr,
            normal,
            occlusion,
            emissive,
        }
    }
}

#[derive(Debug, Serialize, Deserialize)]
pub struct PbrMaterial {
    pub base_color_factor: Vec4,
    pub base_color_texture: Option<RgbaImg>,
    pub metallic_texture: Option<GrayImg>,
    pub metallic_factor: f32,
    pub roughness_texture: Option<GrayImg>,
    pub roughness_factor: f32,
}

impl PbrMaterial {
    pub fn new_from_gltf(pbr_material: easy_gltf::model::PbrMaterial) -> Self {
        let base_color_factor = pbr_material.base_color_factor;
        let base_color_texture = pbr_material
            .base_color_texture
            .map(|m| RgbaImg::new((*m).clone()));
        let metallic_texture = pbr_material
            .metallic_texture
            .map(|m| GrayImg::new((*m).clone()));
        let metallic_factor = pbr_material.metallic_factor;
        let roughness_texture = pbr_material
            .roughness_texture
            .map(|m| GrayImg::new((*m).clone()));
        let roughness_factor = pbr_material.roughness_factor;

        Self {
            base_color_factor,
            base_color_texture,
            metallic_texture,
            metallic_factor,
            roughness_texture,
            roughness_factor,
        }
    }
}

#[derive(Debug, Serialize, Deserialize)]
pub struct NormalMap {
    pub texture: RgbImg,
    pub factor: f32,
}

impl NormalMap {
    pub fn new_from_gltf(normal_map: easy_gltf::model::NormalMap) -> Self {
        let texture = RgbImg::new((*normal_map.texture).clone());
        let factor = normal_map.factor;
        Self { texture, factor }
    }
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Occlusion {
    pub texture: GrayImg,
    pub factor: f32,
}

impl Occlusion {
    pub fn new_from_gltf(occlusion: easy_gltf::model::Occlusion) -> Self {
        let texture = GrayImg::new((*occlusion.texture).clone());
        let factor = occlusion.factor;
        Self { texture, factor }
    }
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Emissive {
    pub texture: Option<RgbImg>,
    pub factor: Vec3,
}

impl Emissive {
    pub fn new_from_gltf(emissive: easy_gltf::model::Emissive) -> Self {
        let texture = emissive.texture.map(|t| RgbImg::new((*t).clone()));
        let factor = emissive.factor;
        Self { texture, factor }
    }
}
