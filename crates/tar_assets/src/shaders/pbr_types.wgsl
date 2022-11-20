#define_import_path tar_pbr::pbr_types

struct MaterialInput {
    base_color_factor: vec4<f32>
    metallic_factor: f32,
    roughness_factor: f32,
    normal_scale: f32,
    occlusion_strength: f32,
    emissive_factor: vec3<f32>
    alpha_cutoff: f32,
    flags: u32,
}

let STANDARD_MATERIAL_FLAGS_BASE_COLOR_TEXTURE_BIT: u32         = 1u;
let STANDARD_MATERIAL_FLAGS_METALLIC_ROUGHNESS_TEXTURE_BIT: u32 = 2u;
let STANDARD_MATERIAL_FLAGS_TWO_COMPONENT_NORMAL_MAP: u32       = 4u;
let STANDARD_MATERIAL_FLAGS_FLIP_NORMAL_MAP_Y: u32              = 8u;
let STANDARD_MATERIAL_FLAGS_OCCLUSION_TEXTURE_BIT: u32          = 16u;
let STANDARD_MATERIAL_FLAGS_EMISSIVE_TEXTURE_BIT: u32           = 32u;
let STANDARD_MATERIAL_FLAGS_ALPHA_MODE_OPAQUE: u32              = 64u;
let STANDARD_MATERIAL_FLAGS_ALPHA_MODE_MASK: u32                = 128u;
let STANDARD_MATERIAL_FLAGS_ALPHA_MODE_BLEND: u32               = 256u;
let STANDARD_MATERIAL_FLAGS_DOUBLE_SIDED_BIT: u32               = 512u;
let STANDARD_MATERIAL_FLAGS_UNLIT_BIT: u32                      = 1024u;