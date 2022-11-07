use std::{collections::HashMap, hash};
use std::path::Path;

use gltf::json::extensions::material;
use image::{
    RgbImage,
    RgbaImage,
    GrayImage,
    Rgb, DynamicImage
};

use serde::{Serialize, Deserialize};

use crate::ExportError;

type ProjPath = String;


#[derive(Serialize, Deserialize)]
/// Model to be saved and loaded to and from disk
pub struct Model {
    pub name: String,
    pub vertices: Vec<Vertex>,
    pub indices: Vec<usize>,
    pub material: Material,
}

#[derive(Serialize, Deserialize)]
/// Contains a position, normal and texture coordinates vectors.
pub struct Vertex {
    /// Position
    pub position: [f32; 3],

    /// Normalized normal
    pub normal: [f32; 3],

    /// Tangent normal 
    /// The 'w' component indicates the direction of the vector
    /// 
    /// Note: to claculate the bitangent do: `cross(normal, tangent) * tangent.w`
    pub tangent: [f32; 4],

    /// Texture coordinates (UV)
    pub tex_coords: [f32; 2],
}

#[derive(Serialize, Deserialize)]
/// Contains material properties of models.
pub struct Material {
    /// Parameter values that define the metallic-roughness material model from
    /// Physically-Based Rendering (PBR) methodology.
    pub pbr: PbrMaterial,

    /// Defines the normal texture of a material.
    pub normal: Option<NormalMap>,

    /// Defines the occlusion texture of a material.
    pub occlusion: Option<Occlusion>,

    /// The emissive color of the material.
    pub emissive: Emissive,
}

#[derive(Serialize, Deserialize)]
/// A set of parameter values that are used to define the metallic-roughness
/// material model from Physically-Based Rendering (PBR) methodology.
pub struct PbrMaterial {
    /// The `base_color_factor` contains scaling factors for the red, green,
    /// blue and alpha component of the color. If no texture is used, these
    /// values will define the color of the whole object in **RGB** color space.
    pub base_color_factor: [f32; 4],

    /// The `base_color_texture` is the main texture that will be applied to the
    /// object.
    ///
    /// The texture contains RGB(A) components in **sRGB** color space.
    pub base_color_texture: Option<RgbaImageRef>,

    /// Contains the metalness value
    pub metallic_texture: Option<GrayImageRef>,

    /// `metallic_factor` is multiply to the `metallic_texture` value. If no
    /// texture is given, then the factor define the metalness for the whole
    /// object.
    pub metallic_factor: f32,

    /// Contains the roughness value
    pub roughness_texture: Option<GrayImageRef>,

    /// `roughness_factor` is multiply to the `roughness_texture` value. If no
    /// texture is given, then the factor define the roughness for the whole
    /// object.
    pub roughness_factor: f32,
}

#[derive(Serialize, Deserialize)]
/// Defines the normal texture of a material.
pub struct NormalMap {
    /// A tangent space normal map.
    /// The texture contains RGB components in linear space. Each texel
    /// represents the XYZ components of a normal vector in tangent space.
    ///
    /// * Red [0 to 255] maps to X [-1 to 1].
    /// * Green [0 to 255] maps to Y [-1 to 1].
    /// * Blue [128 to 255] maps to Z [1/255 to 1].
    ///
    /// The normal vectors use OpenGL conventions where +X is right, +Y is up,
    /// and +Z points toward the viewer.
    pub texture: RgbImageRef,

    /// The `normal_factor` is the normal strength to be applied to the
    /// texture value.
    pub factor: f32,
}

#[derive(Serialize, Deserialize)]
/// Defines the occlusion texture of a material.
pub struct Occlusion {
    /// The `occlusion_texture` refers to a texture that defines areas of the
    /// surface that are occluded from light, and thus rendered darker.
    pub texture: GrayImageRef,

    /// The `occlusion_factor` is the occlusion strength to be applied to the
    /// texture value.
    pub factor: f32,
}

#[derive(Serialize, Deserialize)]
/// The emissive color of the material.
pub struct Emissive {
    /// The `emissive_texture` refers to a texture that may be used to illuminate parts of the
    /// model surface: It defines the color of the light that is emitted from the surface
    pub texture: Option<RgbImageRef>,

    /// The `emissive_factor` contains scaling factors for the red, green and
    /// blue components of this texture.
    pub factor: [f32; 3],
}

#[derive(Serialize, Deserialize)]
/// A reference to an Rgb image
pub struct ImageRef {
    /// Name of an image
    pub name: String,
    /// Relative path to an image
    pub location: Option<ProjPath>,
}

type RgbImageRef = ImageRef;
type GrayImageRef = ImageRef;
type RgbaImageRef = ImageRef;
/// saves a model to the the path specified in target (relative to base directory) or the base directory
pub fn save_model(model: Model, images: HashMap<String, DynamicImage>, target: Option<ProjPath>)-> Result<(), ExportError> {
    let buf = rmp_serde::to_vec(&model)?;
    let path = if let Some(p) = target {
        get_abs_path(p)?
    }
    else {
        get_abs_path(model.name)?
    };

    std::fs::write(path, buf);

    Ok(())
}


fn get_abs_path(local_path: ProjPath) -> std::io::Result<&'static std::path::Path> {
    Ok(Path::new(&Path::new(&std::env::current_dir()?).join(local_path)))
}


pub fn load_model(path: ProjPath) -> Result<(Model, HashMap<String, DynamicImage>), ExportError> {
    let model = rmp_serde::from_slice(std::fs::read(path)?.as_slice())?;

    let images = load_images_from_model(&model)?;

    Ok((model, images))
}

pub fn load_images_from_model(model: &Model) -> Result<HashMap<String, DynamicImage>, ExportError> {
    let mut images = HashMap::new();

    let material = &model.material;
    let pbr = &material.pbr;
    if let Some(bt) = pbr.base_color_texture {
        images.insert(bt.name, load_local_image(bt.location)?);
    }
    if let Some(i) = pbr.metallic_texture {
        images.insert(i.name, load_local_image(i.location)?);
    }
    if let Some(i) = pbr.roughness_texture {
        images.insert(i.name, load_local_image(i.location)?);
    }

    if let Some(normal) = material.normal {
        let i = normal.texture;
        images.insert(i.name, load_local_image(i.location)?);
    }

    let emissive = material.emissive;
    if let Some(i) = emissive.texture {
        images.insert(i.name, load_local_image(i.location)?);
    }


    Ok(images)
}

pub fn load_local_image(local_path: Option<String>) -> Result<DynamicImage, ExportError> {
    let path = get_abs_path(local_path.ok_or(ExportError::MissingPath)?)?;

    image::open(path).map_err(|e| ExportError::Image { source: e })
}