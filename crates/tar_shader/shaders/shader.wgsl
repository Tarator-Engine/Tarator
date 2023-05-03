// ----- Consts ----- 
const PI: f32 = 3.14159265359;

// ----- General data ----- 

struct UniformData {
    /// ambient color
    ambient: vec4<f32>,
    /// view matrix
    view: mat4x4<f32>,
    proj: mat4x4<f32>,
    camera_pos: vec4<f32>,
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

struct MaterialData {
    albedo: vec4<f32>,
    emissive: vec3<f32>,
    roughness: f32,
    metallic: f32,
    reflectance: f32,
    // occlusion: f32,
    flags: u32,
    texture_enable: u32,
}

/// get specified flag from data
fn extract_material_flag(flag: u32) -> bool {
    return bool(material_uniform.flags & flag);
}

/// checks if a texture is enabled
fn extract_texture_enable(texture: u32) -> bool {
    return bool(material_uniform.texture_enable & texture);
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
const TEXTURE_OCCLUSION: u32 =      0x0000040u;

fn get_albedo_texture(coords: vec2<f32>) -> vec4<f32> {
    return textureSample(albedo_tex, primary_sampler, coords);
}

fn get_normal_texture(coords: vec2<f32>) -> vec3<f32> {
    return textureSample(normal_tex, primary_sampler, coords).rgb;
}

fn get_roughness_texture(coords: vec2<f32>) -> f32 {
    return textureSample(roughness_tex, primary_sampler, coords).r;
}

fn get_metallic_texture(coords: vec2<f32>) -> f32 {
    return textureSample(metallic_tex, primary_sampler, coords).r;
}

fn get_emissive_texture(coords: vec2<f32>) -> vec3<f32> {
    return textureSample(emissive_tex, primary_sampler, coords).rgb;
}

fn get_occlusion_texture(coords: vec2<f32>) -> f32 {
    return textureSample(occlusion_tex, primary_sampler, coords).r;
}


// ----- Vertex Data ----- 

struct Vertex {
    @location(0) position: vec3<f32>,
    @location(1) normal: vec3<f32>,
    @location(2) tex_coords: vec2<f32>,
    @location(3) tangent: vec4<f32>,
}

struct Instance {
    @location(4) model_matrix_0: vec4<f32>,
    @location(5) model_matrix_1: vec4<f32>,
    @location(6) model_matrix_2: vec4<f32>,
    @location(7) model_matrix_3: vec4<f32>,
}


struct VertexOutput {
    @builtin(position) position: vec4<f32>,
    @location(0) tex_coords: vec2<f32>,
    @location(1) world_position: vec4<f32>,
    @location(2) normal: vec3<f32>,
    @location(3) tangent: vec4<f32>,
    @location(4) debug: vec3<f32>, 
    @location(5) use_dbg: f32,
};


// ----- Vertex shader ----- 

@vertex
fn vs_main(
    vertex: Vertex,
    instance: Instance,
) -> VertexOutput {

    // the steps to homogeneous coordinates (coordinates on screen) are:
    // model -> world -> camera -> homogeneous
    //     model     view      proj

    // when combining matrices note the correct order
    // model_view_proj = proj * view * model
    
    // in the case of this code the latter two are calculated seperately
    // because we need the position in view coordinates

    let model_matrix = mat4x4<f32>(
        instance.model_matrix_0,
        instance.model_matrix_1,
        instance.model_matrix_2,
        instance.model_matrix_3,
    );

    let model_view = uniforms.view * model_matrix;
    let model_view_proj = uniforms.proj * model_view;

    let position_vec4 = vec4<f32>(vertex.position, 1.0);

    var out: VertexOutput;
    out.world_position = model_view * position_vec4;
    out.normal = vertex.normal;
    out.tangent = vertex.tangent;
    out.tex_coords = vertex.tex_coords;
    out.position = model_view_proj * position_vec4;
    return out;
}

// ----- Fragment Data ----- 

// global frame data
@group(0) @binding(0)
var primary_sampler: sampler;
@group(0) @binding(1)
var<uniform> uniforms: UniformData; 
@group(0) @binding(2)
var<storage> directional_lights: array<DirectionalLight>;

// material specific data
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
@group(1) @binding(6)
var occlusion_tex: texture_2d<f32>;  


// ----- Fragment shader ----- 

@fragment
fn fs_main(vs_out: VertexOutput) -> @location(0) vec4<f32> {
    if extract_material_flag(FLAGS_UNLIT) {
        discard;
    }
    if vs_out.use_dbg != 0.0 {
        return vec4<f32>(abs(vs_out.debug.r), abs(vs_out.debug.g), abs(vs_out.debug.b), 1.0);
    }

    var n = vs_out.normal;
    let v = normalize(uniforms.camera_pos - vs_out.position).xyz;

    var albedo: vec4<f32>;

    if extract_material_flag(FLAGS_ALBEDO_ACTIVE) {
        if extract_texture_enable(TEXTURE_ALBEDO) {
            albedo = get_albedo_texture(vs_out.tex_coords);
        } else {
            albedo = vec4<f32>(1.0);
        }
    } else {
        albedo = vec4<f32>(0.0, 0.0, 0.0, 1.0);
    }
    albedo *= material_uniform.albedo;

    var metallic: f32;
    if extract_texture_enable(TEXTURE_METALLIC) {
        metallic = get_metallic_texture(vs_out.tex_coords);
    } else {
        metallic = 1.0;
    }
    metallic *= material_uniform.metallic;

    var roughness: f32;
    if extract_texture_enable(TEXTURE_ROUGHNESS) {
        roughness = get_roughness_texture(vs_out.tex_coords);
    } else {
        roughness = 1.0;
    }
    roughness *= material_uniform.roughness;

    var normal: vec3<f32>;
    if extract_texture_enable(TEXTURE_NORMAL) {
        normal = get_normal_texture(vs_out.tex_coords);
    } else {
        normal = vec3<f32>(1.0);
    }
    n *= normal;

    // var occlusion: f32;
    // if extract_texture_enable(TEXTURE_OCCLUSION) {
    //     occlusion = get_occlusion_texture(vs_out.tex_coords);
    // } else {
    //     occlusion = 1.0;
    // }
    // occlusion *= material_uniform.occlusion;

    var l_o = vec3<f32>(0.0);
    for (var i = 0; i < i32(arrayLength(&directional_lights)); i += 1) {
        let light = directional_lights[i];
        let l = normalize(light.direction - vs_out.world_position.xyz);
        let h = normalize(v + l);

        let distance = length(light.direction - vs_out.world_position.xyz);
        let attenuation = 1.0 / (distance * distance);
        let radiance = light.color * attenuation;

        var f0 = vec3<f32>(0.04);
        f0 = mix(f0, albedo.rgb, metallic);
        let f = fresnel_schlick(max(dot(h, v), 0.0), f0);

        let ndf = distribution_ggx(n, h, roughness);
        let g = geometry_smith(n, v, l, roughness);

        let numerator = ndf * g * f;
        let denominator = 4.0 * max(dot(n, v), 0.0) * max(dot(n, l), 0.0) + 0.0001;
        let specular = numerator / denominator;

        let k_s = f;
        var k_d = vec3<f32>(1.0) - k_s;

        k_d *= 1.0 - metallic;

        let n_dot_l = max(dot(n, l), 0.0);
        l_o += (k_d * albedo.rgb / PI + specular) * radiance * n_dot_l;
    }

    let ambient = vec3<f32>(0.03) * albedo.rgb; // * occlusion;

    var color = ambient + l_o;

    color = color / (color + vec3<f32>(1.0));
    color = pow(color, vec3<f32>(1.0 / 2.2));

    return vec4<f32>(color, albedo.a);
}

// ----- PBR calculations ----- 

fn distribution_ggx(n: vec3<f32>, h: vec3<f32>, roughness: f32) -> f32 {
    let a = roughness * roughness;
    let a2 = a * a;
    let n_dot_h = max(dot(n, h), 0.0);
    let n_dot_h2 = n_dot_h * n_dot_h;

    let nom = a2;
    var denom = (n_dot_h2 * (a2 - 1.0) + 1.0);
    denom = PI * denom * denom;

    return nom / denom;
}

fn geometry_schlick_ggx(n_dot_v: f32, roughness: f32) -> f32 {
    let r = (roughness + 1.0);
    let k = (r * r) / 8.0;

    let num = n_dot_v;
    let denom = n_dot_v * (1.0 - k) + k;

    return num / denom;
}

fn geometry_smith(n: vec3<f32>, v: vec3<f32>, l: vec3<f32>, roughness: f32) -> f32 {
    let n_dot_v = max(dot(n, v), 0.0);
    let n_dot_l = max(dot(n, l), 0.0);
    let ggx1 = geometry_schlick_ggx(n_dot_v, roughness);
    let ggx2 = geometry_schlick_ggx(n_dot_l, roughness);

    return ggx1 * ggx2;
}

fn fresnel_schlick(cos_theta: f32, f0: vec3<f32>) -> vec3<f32> {
    return f0 + (1.0 - f0) * pow(clamp(1.0 - cos_theta, 0.0, 1.0), 5.0);
}