# Uniform Descriptions
These are the descriptions of the different inputs to the shader

Vertex:
    position: vec3<f32>
    normal: vec3<f32>
    tangent: vec4<f32>
    tex_coord_0: vec2<f32>
    tex_coord_1: vec2<f32>
    color_0: vec4<f32>
    joints_0: vec4<u16>
    weights_0: vec4<f32>

Camera:
    view_pos: vec4<f32>
    view_proj: mat4x4<f32>

Light:
    position: vec3<f32>
    color: vec3<f32>

MaterialInput:
    base_color_factor: vec4<f32>
    metallic_factor: f32,
    roughness_factor: f32,
    normal_scale: f32,
    occlusion_strength: f32,
    emissive_factor: vec3<f32>
    alpha_cutoff: f32,