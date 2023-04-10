use image::DynamicImage;
use tar_types::{Vec3, Vec4};

use super::texture::{GrayTexture, RgbaTexture, Texture};

pub struct Material {
    pub pbr: PbrMaterial,
    pub normal: Option<NormalMap>,
    pub occlusion: Option<Occlusion>,
    pub emissive: Emissive,
}
impl Material {
    pub fn from_stored(
        stored: tar_res::model::material::Material,
        device: &wgpu::Device,
        queue: &wgpu::Queue,
    ) -> Self {
        Self {
            pbr: PbrMaterial::from_stored(stored.pbr, device, queue),
            normal: stored
                .normal
                .map(|n| NormalMap::from_stored(n, device, queue)),
            occlusion: stored
                .occlusion
                .map(|o| Occlusion::from_stored(o, device, queue)),
            emissive: Emissive::from_stored(stored.emissive, device, queue),
        }
    }
}

pub struct PbrMaterial {
    pub base_color_factor: Vec4,
    pub base_color_texture: Option<RgbaTexture>,
    pub metallic_texture: Option<GrayTexture>,
    pub metallic_factor: f32,
    pub roughness_texture: Option<GrayTexture>,
    pub roughness_factor: f32,
}

impl PbrMaterial {
    pub fn from_stored(
        stored: tar_res::model::material::PbrMaterial,
        device: &wgpu::Device,
        queue: &wgpu::Queue,
    ) -> Self {
        Self {
            base_color_factor: stored.base_color_factor,
            base_color_texture: stored.base_color_texture.map(|img| {
                RgbaTexture::from_image(
                    device,
                    queue,
                    &DynamicImage::ImageRgba8(img),
                    "base_color_texture",
                )
            }),
            metallic_texture: stored.metallic_texture.map(|img| {
                GrayTexture::from_image(
                    device,
                    queue,
                    &DynamicImage::ImageLuma8(img),
                    "metallic_texture",
                )
            }),
            metallic_factor: stored.metallic_factor,
            roughness_texture: stored.roughness_texture.map(|img| {
                GrayTexture::from_image(
                    device,
                    queue,
                    &DynamicImage::ImageLuma8(img),
                    "roughness_texture",
                )
            }),
            roughness_factor: stored.roughness_factor,
        }
    }
}

pub struct NormalMap {
    pub texture: RgbaTexture,
    pub factor: f32,
}

impl NormalMap {
    pub fn from_stored(
        stored: tar_res::model::material::NormalMap,
        device: &wgpu::Device,
        queue: &wgpu::Queue,
    ) -> Self {
        Self {
            texture: RgbaTexture::from_image(
                device,
                queue,
                &DynamicImage::ImageRgb8(stored.texture),
                "normal_texture",
            ),
            factor: stored.factor,
        }
    }
}

pub struct Occlusion {
    pub texture: GrayTexture,
    pub factor: f32,
}

impl Occlusion {
    pub fn from_stored(
        stored: tar_res::model::material::Occlusion,
        device: &wgpu::Device,
        queue: &wgpu::Queue,
    ) -> Self {
        Self {
            texture: GrayTexture::from_image(
                device,
                queue,
                &DynamicImage::ImageLuma8(stored.texture),
                "occlusion_texture",
            ),
            factor: stored.factor,
        }
    }
}

pub struct Emissive {
    pub texture: Option<RgbaTexture>,
    pub factor: Vec3,
}

impl Emissive {
    pub fn from_stored(
        stored: tar_res::model::material::Emissive,
        device: &wgpu::Device,
        queue: &wgpu::Queue,
    ) -> Self {
        Self {
            texture: stored.texture.map(|img| {
                RgbaTexture::from_image(
                    device,
                    queue,
                    &DynamicImage::ImageRgb8(img),
                    "emissive_texture",
                )
            }),
            factor: stored.factor,
        }
    }
}
