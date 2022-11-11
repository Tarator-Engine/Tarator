use serde::{Serialize, Deserialize};

use uuid::Uuid;

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
    pub vertecies: Option<wgpu::Buffer>,
    pub indecies: Option<wgpu::Buffer>,
    pub num_indecies: u32,
    pub material: u32,
}

/// Contains material properties of models.
pub struct RawMaterial {
    /// diffuse texture of a normal: this is also sometimes called base color
    pub diffuse_tex: Option<RawTexture>,

    /// normal texture of a material.
    pub normal_tex: Option<RawTexture>,

    /// metallicness texture of a material.
    pub metallic_tex: Option<RawTexture>,

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


#[derive(Debug, Serialize, Deserialize)]
pub struct StoreModel {
    pub id: Uuid,
    pub meshes: Vec<StoreMesh>,
    pub materials: Vec<StoreMaterial>,
    pub instances: Vec<StoreInstance>,
    pub instance_num: u32,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct StoreMesh {
    pub vertecies: Vec<StoreVertex>,
    pub indecies: Vec<u32>,
    pub num_indecies: u32,
    pub material: u32,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct StoreVertex {
    pub position: [f32; 3],
    pub normal: [f32; 3],
    pub tangent: [f32; 4],
    pub tex_coords: [f32; 2],
}

#[derive(Debug, Serialize, Deserialize)]
pub struct StoreMaterial {
    /// diffuse texture of a normal: this is also sometimes called base color
    pub diffuse_tex: std::path::PathBuf,

    /// normal texture of a material.
    pub normal_tex: std::path::PathBuf,

    /// metallicness texture of a material.
    pub metallic_tex: std::path::PathBuf,

    /// roughness texture of a material
    pub roughness_tex: std::path::PathBuf,

    /// Occlusion Texture of a material
    pub occlusion_tex: std::path::PathBuf,

    /// The emissive color of the material.
    pub emissive_tex: std::path::PathBuf,

    /// The `base_color_factor` contains scaling factors for the red, green,
    /// blue and alpha component of the color. If no texture is used, these
    /// values will define the color of the whole object in **RGB** color space.
    pub base_color_factor: [f32; 4],

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

#[derive(Debug, Serialize, Deserialize)]
pub struct StoreInstance {
    pub position: [f32; 3],
    pub rotation: [f32; 4],
}

pub struct Instance {
    pub position: cgmath::Vector3<f32>,
    pub rotation: cgmath::Quaternion<f32>,
}