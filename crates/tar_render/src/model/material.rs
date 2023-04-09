use tar_types::{Vec3, Vec4};

use super::texture::{GrayTexture, RgbTexture};

pub struct Material {
    pub pbr: PbrMaterial,
}

pub struct PbrMaterial {
    pub base_color_factor: Vec4,
    pub base_color_texture: Option<RgbTexture>,
    pub metallic_texture: Option<GrayTexture>,
    pub metallic_factor: f32,
    pub roughness_texture: Option<GrayTexture>,
    pub roughness_factor: f32,
}

pub struct NormalMap {
    pub texture: RgbTexture,
    pub factor: f32,
}

pub struct Occlusion {
    pub texture: GrayTexture,
    pub factor: f32,
}

pub struct Emissive {
    pub texture: Option<RgbTexture>,
    pub factor: Vec3,
}
