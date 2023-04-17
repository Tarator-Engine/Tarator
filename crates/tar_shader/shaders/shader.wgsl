// ----- Consts ----- 
const PI: f32 = 3.14159265359;

// ----- General data ----- 

struct UniformData {
    /// ambient color
    ambient: vec4<f32>,
    /// view matrix
    view: mat4x4<f32>,
    view_proj: mat4x4<f32>,
    object_transform: mat4x4<f32>,
}

struct DirectionalLight {
    color: vec3<f32>,
    padding: f32,
    direction: vec3<f32>,
    padding2: f32,
}

struct PointLight {
    color: vec3<f32>,
    position: vec3<f32>,
}

struct PixelData {
    albedo: vec4<f32>,
    diffuse_color: vec3<f32>,
    roughness: f32,
    normal: vec3<f32>,
    metallic: f32,
    emissive: vec3<f32>,
    reflectance: f32,
    f0: vec3<f32>,
    material_flags: u32,
}

fn get_pixel_data(material: MaterialData, vs_out: VertexOutput) -> PixelData {
    var pixel: PixelData;

    let coords = vs_out.tex_coords;

    // ----- ALBEDO ----- 

    if extract_material_flag(material.flags, FLAGS_ALBEDO_ACTIVE) {
        if extract_texture_enable(material.texture_enable, TEXTURE_ALBEDO) {
            pixel.albedo = get_texture(TEXTURE_ALBEDO, coords);
        } else {
            pixel.albedo = vec4<f32>(1.0);
        }
    } else {
        pixel.albedo = vec4<f32>(0.0, 0.0, 0.0, 1.0);
    }

    pixel.albedo *= material.albedo;

    // TODO!: several potential stops

    // ----- NORMAL ----- 

    if extract_texture_enable(material.texture_enable, TEXTURE_NORMAL) {
        let texture_read = get_texture(TEXTURE_NORMAL, coords);

        var normal = normalize(texture_read.rgb * 2.0 - 1.0);

        let normal_norm = normalize(vs_out.normal);
        let tangent_norm = normalize(vs_out.tangent.xyz);
        let bitangent = cross(normal_norm, tangent_norm) * vs_out.tangent.w;

        let tbn = mat3x3(tangent_norm, bitangent, normal_norm);

        pixel.normal = tbn * normal;
    } else {
        pixel.normal = vs_out.normal;
    }


    // ----- Metallic -----

    if extract_texture_enable(material.texture_enable, TEXTURE_METALLIC) {
        // TODO!: is the data structured like this? 
        pixel.metallic = get_texture(TEXTURE_METALLIC, coords);
    } else {
        pixel.metallic = 1.0;
    }
    pixel.metallic *= material.metallic;


    // ----- Roughness -----

    var perceptual_roughness: f32;

    if extract_texture_enable(material.texture_enable, TEXTURE_ROUGHNESS) {
        // TODO!: is the data structured like this? 
        perceptual_roughness = get_texture(TEXTURE_ROUGHNESS, coords);

    } else {
        perceptual_roughness = 1.0;
    }
    perceptual_roughness *= material.roughness;

    
    // ----- Reflectance -----

    if extract_texture_enable(material.texture_enable, TEXTURE_REFLECTANCE) {
        // TODO!: is the data structured like this? 
        pixel.reflectance = get_texture(TEXTURE_REFLECTANCE, coords);
    } else {
        pixel.reflectance = 1.0;
    }
    pixel.reflectance *= material.reflectance;

    
    // ----- Emissive -----

    if extract_texture_enable(material.texture_enable, TEXTURE_EMISSIVE) {
        // TODO!: is the data structured like this? 
        pixel.emissive = get_texture(TEXTURE_EMISSIVE, coords);
    } else {
        pixel.emissive = vec3<f32>(1.0);
    }
    pixel.emissive *= material.emissive;


    // ----- Computations -----

    // compute the diffuse color based on the metallicness of the material
    pixel.diffuse_color = pixel.albedo.rgb * (1.0 - pixel.metallic);

    // Assumes an interface from air to an IOR of 1.5 for dielectrics
    // compute dielectic f0
    let reflectance = 0.16 * pixel.reflectance * pixel.reflectance;
    // compute f0
    pixel.f0 = pixel.albedo.rgb * pixel.metallic + (reflectance * (1.0 - pixel.metallic));


    // compute roughness from perceptual roughness by squaring it
    pixel.roughness = perceptual_roughness * perceptual_roughness;

    return pixel;
}

struct MaterialData {
    albedo: vec4<f32>,
    emissive: vec3<f32>,
    roughness: f32,
    metallic: f32,
    reflectance: f32,
    flags: u32,
    texture_enable: u32,
}

fn brdf_d_ggx(noh: f32, a: f32) -> f32 {
    let a2 = a * a;
    let f = (noh * a2 - noh) * noh + 1.0;
    return a2 / (PI * f * f);
}

fn brdf_f_schlick_vec3(u: f32, f0: vec3<f32>, f90: f32) -> vec3<f32> {
    return f0 + (f90 - f0) * pow(1.0 - u, 5.0);
}

fn brdf_v_smith_ggx_correlated(nov: f32, nol: f32, a: f32) -> f32 {
    let a2 = a * a;
    let ggxl = nov * sqrt((-nol * a2 + nol) * nol + a2);
    let ggxv = nol * sqrt((-nov * a2 + nov) * nov + a2);
    return 0.5 / (ggxl + ggxv);
}

fn brdf_fd_lambert() -> f32 {
    return 1.0 / PI;
}

fn surface_shading(light: DirectionalLight, pixel: PixelData, view_pos: vec3<f32>) -> vec3<f32> {
    let view_mat3 = mat3x3<f32>(uniforms.view[0].xyz, uniforms.view[1].xyz, uniforms.view[2].xyz);
    let l = normalize(view_mat3 * -light.direction);

    let n = pixel.normal;
    let h = normalize(view_pos + l);

    let nov = abs(dot(n, view_pos)) + 0.00001;
    let nol = clamp(dot(n, l), 0.0, 1.0);
    let noh = clamp(dot(n, h), 0.0, 1.0);
    let loh = clamp(dot(l, h), 0.0, 1.0);

    let f90 = clamp(dot(pixel.f0, vec3<f32>(50.0 * 0.33)), 0.0, 1.0);

    let d = brdf_d_ggx(noh, pixel.roughness);
    let f = brdf_f_schlick_vec3(loh, pixel.f0, f90);
    let v = brdf_v_smith_ggx_correlated(nov, nol, pixel.roughness);

    // TODO!: figure out how they generate their lut
    let energy_comp = 1.0;

    // specular
    let fr = (d * v) * f;
    // diffuse
    let fd = pixel.diffuse_color * brdf_fd_lambert();

    let color = fd + fr * energy_comp;

    let light_attenuation = 1.0;

    // TODO!: figure out if it is ok to leave out occlusion
    return (color * light.color) * (light_attenuation * nol);
}

/// get specified flag from data
fn extract_material_flag(data: u32, flag: u32) -> bool {
    return bool(data & flag);
}

/// checks if a texture is enabled
fn extract_texture_enable(data: u32, texture: u32) -> bool {
    return bool(data & texture);
}

// TODO!: FLAGS (material and textures)

// ----- Material Flags ----- 
const FLAGS_ALBEDO_ACTIVE: u32 =    0x0000001u;
const FLAGS_UNLIT: u32 =            0x0000002u;

// ----- TEXTURES ----- 
const TEXTURE_ALBEDO: u32 =         0x0000001u;
const TEXTURE_NORMAL: u32 =         0x0000002u;
const TEXTURE_ROUGHNESS: u32 =      0x0000004u;
const TEXTURE_METALLIC: u32 =       0x0000008u;
const TEXTURE_REFLECTANCE: u32 =    0x0000010u;
const TEXTURE_EMISSIVE: u32 =       0x0000020u;

fn get_texture(texture: u32, coords: vec2<f32>) -> vec4<f32> {
    // MAKE SURE these stay the same as above
    switch texture {
        case 0x0000001u: {
            return textureSample(albedo_tex, primary_sampler, coords);
        }
        case 0x0000002u: {
            return textureSample(normal_tex, primary_sampler, coords);
        }
        case 0x0000004u: {
            return textureSample(roughness_tex, primary_sampler, coords);
        }
        case 0x0000008u: {
            return textureSample(metallic_tex, primary_sampler, coords);
        }
        // case 0x0000010u: {
        //     return textureSample(reflectance_tex, primary_sampler, coords);
        // }
        case 0x0000020u: {
            return textureSample(emissive_tex, primary_sampler, coords);
        }
    }
}


// ----- Vertex Data ----- 

struct Vertex {
    @location(0) position: vec3<f32>,
    @location(1) normal: vec3<f32>,
    @location(2) tangent: vec4<f32>,
    @location(3) tex_coords: vec2<f32>,
}

struct VertexOutput {
    @builtin(position) position: vec4<f32>,
    @location(0) tex_coords: vec2<f32>,
    @location(1) view_position: vec4<f32>,
    @location(2) normal: vec3<f32>,
    @location(3) tangent: vec4<f32>,
};


// ----- Vertex shader ----- 

fn mat3_inv_scale_squared(transform: mat3x3<f32>) -> vec3<f32> {
    return vec3<f32>(
        1.0 / dot(transform[0].xyz, transform[0].xyz),
        1.0 / dot(transform[1].xyz, transform[1].xyz),
        1.0 / dot(transform[2].xyz, transform[2].xyz)
    );
}


@vertex
fn vs_main(
    vertex: Vertex,
) -> VertexOutput {

    let model_view = uniforms.view * uniforms.object_transform;
    let model_view_proj = uniforms.view_proj * uniforms.object_transform;


    let position_vec4 = vec4<f32>(vertex.position, 1.0);
    let mv_mat3 = mat3x3<f32>(model_view[0].xyz, model_view[1].xyz, model_view[2].xyz);


    let inv_scale_sq = mat3_inv_scale_squared(mv_mat3);


    var out: VertexOutput;
    out.view_position = model_view * position_vec4;
    out.normal = normalize(mv_mat3 * (inv_scale_sq * vertex.normal));
    out.tangent = vec4<f32>(normalize(mv_mat3 * (inv_scale_sq * vertex.tangent.xyz)), vertex.tangent.w);
    out.tex_coords = vertex.tex_coords;
    out.position = vec4<f32>(vertex.position, 1.0);
    return out;
}


// ----- Fragment Data ----- 

@group(0) @binding(0)
var primary_sampler: sampler;
@group(0) @binding(1)
var<uniform> uniforms: UniformData; 
@group(0) @binding(2)
var<storage> directional_lights: array<DirectionalLight>;

@group(1) @binding(0)
var<uniform> material_uniform: MaterialData;
@group(1) @binding(1)
var albedo_tex: texture_2d<f32>;
@group(1) @binding(2)
var normal_tex: texture_2d<f32>;
@group(1) @binding(3)
var roughness_tex: texture_2d<f32>;
@group(1) @binding(4)
var metallic_tex: texture_2d<f32>;
// @group(1) @binding(5)
// var reflectance_tex: texture_2d<f32>;
@group(1) @binding(5)
var emissive_tex: texture_2d<f32>;


// ----- Fragment shader ----- 

@fragment
fn fs_main(vs_out: VertexOutput) -> @location(0) vec4<f32> {
    let material = material_uniform;

    let pixel = get_pixel_data(material, vs_out);

    if extract_material_flag(material.flags, FLAGS_UNLIT) {
        return pixel.albedo;
    }

    let v = -normalize(vs_out.view_position.xyz);

    var color = pixel.emissive.rgb;

    for (var i = 0; i < i32(arrayLength(&directional_lights)); i += 1) {
        let light = directional_lights[i];
        color += surface_shading(light, pixel, v);
    }

    let ambient = uniforms.ambient * pixel.albedo;
    let both = vec4<f32>(color, pixel.albedo.a);
    return max(ambient, both);
} 