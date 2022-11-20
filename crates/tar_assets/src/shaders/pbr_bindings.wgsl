#define_import_path tar_pbr::pbr_bindings

#import tar_pbr::pbr_types

@group(?) @binding(0)
var<uniform> material: MaterialInput;
@group(?) @binding(1)
var base_color_tex: texture_2d<f32>;
@group(?) @binding(2)
var base_color_sampler: sampler;
@group(?) @binding(3)
var metallix_roughness_tex: texture_2d<f32>;
@group(?) @binding(4)
var metallix_roughness_sampler: sampler;
@group(?) @binding(5)
var normal_tex: texture_2d<f32>;
@group(?) @binding(6)
var normal_sampler: sampler;
@group(?) @binding(7)
var occlusion_tex: texture_2d<f32>;
@group(?) @binding(8)
var occlusion_sampler: sampler;
@group(?) @binding(9)
var emissive_tex: texture_2d<f32>;
@group(?) @binding(10)
var emissive_sampler: sampler;

