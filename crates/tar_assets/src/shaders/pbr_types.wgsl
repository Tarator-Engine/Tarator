struct MaterialInput {
    base_color_factor: vec4<f32>
    metallic_factor: f32,
    roughness_factor: f32,
    normal_scale: f32,
    occlusion_strength: f32,
    emissive_factor: vec3<f32>
    alpha_cutoff: f32,
}