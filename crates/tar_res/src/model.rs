use std::num::NonZeroU32;

use bytemuck::Zeroable;
use serde::{Serialize, Deserialize};

use uuid::Uuid;

use crate::Vec3Slice;

use super::{Result, Error};

/// Model to be saved and loaded to and from disk
pub struct RawModel {
    pub id: Uuid,
    pub meshes: Vec<RawMesh>,
    pub materials: Vec<RawMaterial>,
    pub instances: wgpu::Buffer,
    pub instance_num: u32,
}

/// Contains a position, normal and texture coordinates vectors.
pub struct RawMesh {
    pub positions: wgpu::Buffer,
    pub normals: Option<wgpu::Buffer>,
    pub tangents: Option<wgpu::Buffer>,
    pub tex_coords: Option<wgpu::Buffer>,
    pub indices: wgpu::Buffer,
    pub num_indices: u32,
    pub material: u32,
}

/// Contains material properties of models.
pub struct RawMaterial {
    /// diffuse texture of a normal: this is also sometimes called base color
    pub diffuse_tex: Option<RawTexture>,

    /// normal texture of a material.
    pub normal_tex: Option<RawTexture>,

    /// metallicness texture of a material.
    pub occlusion_metallic_roughness_tex: Option<RawTexture>,

    /// roughness texture of a material
    pub roughness_tex: Option<RawTexture>,

    /// Occlusion Texture of a material
    pub occlusion_tex: Option<RawTexture>,

    /// The emissive color of the material.
    pub emissive_tex: Option<RawTexture>,

    /// the info buffer contains extra informatino aobout the
    /// material and fallback values in case some textures
    /// aren't provided
    pub imfo_buffer: wgpu::Buffer,
}

pub struct RawTexture {
    pub texture: wgpu::Texture,
    pub view: wgpu::TextureView,
    pub sampler: wgpu::Sampler,
}

impl RawTexture {
    pub fn diffuse_texture(
        device: &wgpu::Device,
        queue: &wgpu::Queue,
        img: &image::DynamicImage,
        label: Option<&str>,
    ) -> Result<Self>{
        let dimensions = (img.height(), img.width());

        let size = wgpu::Extent3d {
            width: dimensions.0,
            height: dimensions.1,
            depth_or_array_layers: 1,
        };

        let texture = device.create_texture(&wgpu::TextureDescriptor {
            label,
            size,
            mip_level_count: 1,
            sample_count: 1,
            dimension: wgpu::TextureDimension::D2,
            format: wgpu::TextureFormat::Rgba8UnormSrgb,
            usage: wgpu::TextureUsages::TEXTURE_BINDING,
        });

        queue.write_texture(
            wgpu::ImageCopyTexture {
                aspect: wgpu::TextureAspect::All,
                texture: &texture,
                mip_level: 0,
                origin: wgpu::Origin3d::ZERO,
            }, 
            &img.as_rgba8().ok_or(Error::InvalidImage)?, 
            wgpu::ImageDataLayout {
                offset: 0,
                bytes_per_row: NonZeroU32::new(4 * dimensions.0),
                rows_per_image: NonZeroU32::new(dimensions.1),
            },
            size
        );
        let view = texture.create_view(&wgpu::TextureViewDescriptor::default());
        let sampler = device.create_sampler(&wgpu::SamplerDescriptor {
            address_mode_u: wgpu::AddressMode::ClampToEdge,
            address_mode_v: wgpu::AddressMode::ClampToEdge,
            address_mode_w: wgpu::AddressMode::ClampToEdge,
            mag_filter: wgpu::FilterMode::Linear,
            min_filter: wgpu::FilterMode::Nearest,
            mipmap_filter: wgpu::FilterMode::Nearest,
            ..Default::default()
        });
        
        Ok(Self {
            texture,
            view,
            sampler
        })
    }

    pub fn normal_texture(
        device: &wgpu::Device,
        queue: &wgpu::Queue,
        img: &image::DynamicImage,
        label: Option<&str>,
    ) -> Result<Self> {
        let dimensions = (img.height(), img.width());

        let size = wgpu::Extent3d {
            width: dimensions.0,
            height: dimensions.1,
            depth_or_array_layers: 1,
        };

        let texture = device.create_texture(&wgpu::TextureDescriptor {
            label,
            size,
            mip_level_count: 1,
            sample_count: 1,
            dimension: wgpu::TextureDimension::D2,
            format: wgpu::TextureFormat::Rgba8Unorm,
            usage: wgpu::TextureUsages::TEXTURE_BINDING,
        });

        queue.write_texture(
            wgpu::ImageCopyTexture {
                aspect: wgpu::TextureAspect::All,
                texture: &texture,
                mip_level: 0,
                origin: wgpu::Origin3d::ZERO,
            }, 
            &img.as_rgba8().ok_or(Error::InvalidImage)?, 
            wgpu::ImageDataLayout {
                offset: 0,
                bytes_per_row: NonZeroU32::new(4 * dimensions.0),
                rows_per_image: NonZeroU32::new(dimensions.1),
            },
            size
        );
        let view = texture.create_view(&wgpu::TextureViewDescriptor::default());
        let sampler = device.create_sampler(&wgpu::SamplerDescriptor {
            address_mode_u: wgpu::AddressMode::ClampToEdge,
            address_mode_v: wgpu::AddressMode::ClampToEdge,
            address_mode_w: wgpu::AddressMode::ClampToEdge,
            mag_filter: wgpu::FilterMode::Linear,
            min_filter: wgpu::FilterMode::Nearest,
            mipmap_filter: wgpu::FilterMode::Nearest,
            ..Default::default()
        });
        
        Ok(Self {
            texture,
            view,
            sampler
        })
    }

    pub fn occ_met_rou_texture(
        device: &wgpu::Device,
        queue: &wgpu::Queue,
        img: &image::DynamicImage,
        label: Option<&str>,
    ) -> Result<Self> {
        let dimensions = (img.height(), img.width());

        let size = wgpu::Extent3d {
            width: dimensions.0,
            height: dimensions.1,
            depth_or_array_layers: 1,
        };

        let texture = device.create_texture(&wgpu::TextureDescriptor {
            label,
            size,
            mip_level_count: 1,
            sample_count: 1,
            dimension: wgpu::TextureDimension::D2,
            format: wgpu::TextureFormat::Rgba8Unorm,
            usage: wgpu::TextureUsages::TEXTURE_BINDING,
        });

        queue.write_texture(
            wgpu::ImageCopyTexture {
                aspect: wgpu::TextureAspect::All,
                texture: &texture,
                mip_level: 0,
                origin: wgpu::Origin3d::ZERO,
            }, 
            &img.as_rgba8().ok_or(Error::InvalidImage)?, 
            wgpu::ImageDataLayout {
                offset: 0,
                bytes_per_row: NonZeroU32::new(4 * dimensions.0),
                rows_per_image: NonZeroU32::new(dimensions.1),
            },
            size
        );
        let view = texture.create_view(&wgpu::TextureViewDescriptor::default());
        let sampler = device.create_sampler(&wgpu::SamplerDescriptor {
            address_mode_u: wgpu::AddressMode::ClampToEdge,
            address_mode_v: wgpu::AddressMode::ClampToEdge,
            address_mode_w: wgpu::AddressMode::ClampToEdge,
            mag_filter: wgpu::FilterMode::Linear,
            min_filter: wgpu::FilterMode::Nearest,
            mipmap_filter: wgpu::FilterMode::Nearest,
            ..Default::default()
        });
        
        Ok(Self {
            texture,
            view,
            sampler
        })
    }

    pub fn met_rou_texture(
        device: &wgpu::Device,
        queue: &wgpu::Queue,
        img: &image::DynamicImage,
        label: Option<&str>,
    ) -> Result<Self> {
        let dimensions = (img.height(), img.width());

        let size = wgpu::Extent3d {
            width: dimensions.0,
            height: dimensions.1,
            depth_or_array_layers: 1,
        };

        let texture = device.create_texture(&wgpu::TextureDescriptor {
            label,
            size,
            mip_level_count: 1,
            sample_count: 1,
            dimension: wgpu::TextureDimension::D2,
            format: wgpu::TextureFormat::Rgba8Unorm,
            usage: wgpu::TextureUsages::TEXTURE_BINDING,
        });

        queue.write_texture(
            wgpu::ImageCopyTexture {
                aspect: wgpu::TextureAspect::All,
                texture: &texture,
                mip_level: 0,
                origin: wgpu::Origin3d::ZERO,
            }, 
            &img.as_rgba8().ok_or(Error::InvalidImage)?, 
            wgpu::ImageDataLayout {
                offset: 0,
                bytes_per_row: NonZeroU32::new(4 * dimensions.0),
                rows_per_image: NonZeroU32::new(dimensions.1),
            },
            size
        );
        let view = texture.create_view(&wgpu::TextureViewDescriptor::default());
        let sampler = device.create_sampler(&wgpu::SamplerDescriptor {
            address_mode_u: wgpu::AddressMode::ClampToEdge,
            address_mode_v: wgpu::AddressMode::ClampToEdge,
            address_mode_w: wgpu::AddressMode::ClampToEdge,
            mag_filter: wgpu::FilterMode::Linear,
            min_filter: wgpu::FilterMode::Nearest,
            mipmap_filter: wgpu::FilterMode::Nearest,
            ..Default::default()
        });
        
        Ok(Self {
            texture,
            view,
            sampler
        })
    }

    pub fn occlusion_texture(
        device: &wgpu::Device,
        queue: &wgpu::Queue,
        img: &image::DynamicImage,
        label: Option<&str>,
    ) -> Result<Self> {
        let dimensions = (img.height(), img.width());

        let size = wgpu::Extent3d {
            width: dimensions.0,
            height: dimensions.1,
            depth_or_array_layers: 1,
        };

        let texture = device.create_texture(&wgpu::TextureDescriptor {
            label,
            size,
            mip_level_count: 1,
            sample_count: 1,
            dimension: wgpu::TextureDimension::D2,
            format: wgpu::TextureFormat::Rgba8Unorm,
            usage: wgpu::TextureUsages::TEXTURE_BINDING,
        });

        queue.write_texture(
            wgpu::ImageCopyTexture {
                aspect: wgpu::TextureAspect::All,
                texture: &texture,
                mip_level: 0,
                origin: wgpu::Origin3d::ZERO,
            }, 
            &img.as_rgba8().ok_or(Error::InvalidImage)?, 
            wgpu::ImageDataLayout {
                offset: 0,
                bytes_per_row: NonZeroU32::new(4 * dimensions.0),
                rows_per_image: NonZeroU32::new(dimensions.1),
            },
            size
        );
        let view = texture.create_view(&wgpu::TextureViewDescriptor::default());
        let sampler = device.create_sampler(&wgpu::SamplerDescriptor {
            address_mode_u: wgpu::AddressMode::ClampToEdge,
            address_mode_v: wgpu::AddressMode::ClampToEdge,
            address_mode_w: wgpu::AddressMode::ClampToEdge,
            mag_filter: wgpu::FilterMode::Linear,
            min_filter: wgpu::FilterMode::Nearest,
            mipmap_filter: wgpu::FilterMode::Nearest,
            ..Default::default()
        });
        
        Ok(Self {
            texture,
            view,
            sampler
        })
    }

    pub fn emissive_texture(
        device: &wgpu::Device,
        queue: &wgpu::Queue,
        img: &image::DynamicImage,
        label: Option<&str>,
    ) -> Result<Self> {
        let dimensions = (img.height(), img.width());

        let size = wgpu::Extent3d {
            width: dimensions.0,
            height: dimensions.1,
            depth_or_array_layers: 1,
        };

        let texture = device.create_texture(&wgpu::TextureDescriptor {
            label,
            size,
            mip_level_count: 1,
            sample_count: 1,
            dimension: wgpu::TextureDimension::D2,
            format: wgpu::TextureFormat::Rgba8Unorm,
            usage: wgpu::TextureUsages::TEXTURE_BINDING,
        });

        queue.write_texture(
            wgpu::ImageCopyTexture {
                aspect: wgpu::TextureAspect::All,
                texture: &texture,
                mip_level: 0,
                origin: wgpu::Origin3d::ZERO,
            }, 
            &img.as_rgba8().ok_or(Error::InvalidImage)?, 
            wgpu::ImageDataLayout {
                offset: 0,
                bytes_per_row: NonZeroU32::new(4 * dimensions.0),
                rows_per_image: NonZeroU32::new(dimensions.1),
            },
            size
        );
        let view = texture.create_view(&wgpu::TextureViewDescriptor::default());
        let sampler = device.create_sampler(&wgpu::SamplerDescriptor {
            address_mode_u: wgpu::AddressMode::ClampToEdge,
            address_mode_v: wgpu::AddressMode::ClampToEdge,
            address_mode_w: wgpu::AddressMode::ClampToEdge,
            mag_filter: wgpu::FilterMode::Linear,
            min_filter: wgpu::FilterMode::Nearest,
            mipmap_filter: wgpu::FilterMode::Nearest,
            ..Default::default()
        });
        
        Ok(Self {
            texture,
            view,
            sampler
        })
    }
}


#[derive(Debug, Serialize, Deserialize)]
pub struct StoreModel {
    pub id: Uuid,
    pub meshes: Vec<StoreMesh>,
    pub materials: Vec<StoreMaterial>,
    pub instances: Vec<StoreInstance>,
    pub instance_num: usize,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct StoreMesh {
    pub positions: Vec<[f32; 3]>,
    pub normals: Option<Vec<[f32; 3]>>,
    pub tangents: Option<Vec<[f32; 4]>>,
    pub tex_coords: Option<Vec<[f32; 2]>>,
    pub indices: Vec<usize>,
    pub material: usize,
}

// #[derive(Debug, Serialize, Deserialize)]
// pub struct StoreVertex {
//     pub position: [f32; 3],
//     pub normal: [f32; 3],
//     pub tangent: [f32; 4],
//     pub tex_coords: [f32; 2],
// }

#[derive(Debug, Serialize, Deserialize)]
pub struct StoreMaterial {
    /// diffuse texture of a normal: this is also sometimes called base color
    pub diffuse_tex: Option<std::path::PathBuf>,

    /// normal texture of a material.
    pub normal_tex: Option<std::path::PathBuf>,

    /// metallicness texture of a material.
    pub occlusion_metallic_roughness_tex: Option<std::path::PathBuf>,

    /// roughness texture of a material
    pub metallic_roughness_tex: Option<std::path::PathBuf>,

    /// Occlusion Texture of a material
    pub occlusion_tex: Option<std::path::PathBuf>,

    /// The emissive color of the material.
    pub emissive_tex: Option<std::path::PathBuf>,

    /// The `base_color_factor` contains scaling factors for the red, green,
    /// blue and alpha component of the color. If no texture is used, these
    /// values will define the color of the whole object in **RGB** color space.
    pub diffuse_factor: [f32; 4],

    /// `metallic_factor` is multiply to the `metallic_texture` value. If no
    /// texture is given, then the factor define the metalness for the whole
    /// object.
    pub metallic_factor: f32,

    /// `roughness_factor` is multiply to the `roughness_texture` value. If no
    /// texture is given, then the factor define the roughness for the whole
    /// object.
    pub roughness_factor: f32,
    
    /// The `normal_factor` is the normal strength to be applied to the
    /// texture value.
    pub normal_factor: f32,

    /// The `occlusion_factor` is the occlusion strength to be applied to the
    /// texture value.
    pub occlusion_factor: f32,

    /// The `emissive_factor` contains scaling factors for the red, green and
    /// blue components of this texture.
    pub emissive_factor: [f32; 3],
}

#[repr(C)]
#[derive(Clone, Copy, Debug, Serialize, Deserialize, bytemuck::Pod, Zeroable)]
pub struct StoreInstance {
    pub position: [f32; 3],
    pub rotation: [f32; 4],
}

pub struct Instance {
    pub position: cgmath::Vector3<f32>,
    pub rotation: cgmath::Quaternion<f32>,
}

impl From<Instance> for StoreInstance {
    fn from(i: Instance) -> Self {
        StoreInstance { 
            position: i.position.as_slice(), 
            rotation: [i.rotation.v.x, i.rotation.v.y, i.rotation.v.z, i.rotation.s],
        }
    }
}