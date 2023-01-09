//!include pbr_types.wgsl

// these have to match with the definitions in primitive.rs
struct VertexInput {
    @location(0) position: vec3<f32>,
    @location(1) normal: vec3<f32>,
    @location(2) tangent: vec4<f32>,
    @location(3) tex_coord_0: vec2<f32>,
    @location(4) tex_coord_1: vec2<f32>,
    @location(5) color_0: vec4<f32>,
    @location(6) joints_0: vec4<f32>,
    @location(7) weights_0: vec4<f32>,
}
struct InstanceInput {
    @location(8) model_matrix_0: vec4<f32>,
    @location(9) model_matrix_1: vec4<f32>,
    @location(10) model_matrix_2: vec4<f32>,
    @location(11) model_matrix_3: vec4<f32>,
    @location(12) normal_matrix_0: vec3<f32>,
    @location(13) normal_matrix_1: vec3<f32>,
    @location(14) normal_matrix_2: vec3<f32>,
}

struct VertexOutput {
    @builtin(position) gl_Position: vec4<f32>,
    @location(0) v_UV_0: vec2<f32>,
    @location(1) v_UV_1: vec2<f32>,
    @location(2) v_Color: vec4<f32>,
    //!ifdef HAS_NORMALS 
        //!ifdef HAS_TANGENTS
            @location(3) v_TBN_0: vec3<f32>,
            @location(4) v_TBN_1: vec3<f32>,
            @location(5) v_TBN_2: vec3<f32>,
            @location(6) v_Position: vec3<f32>
        //!else
            @location(3) v_Normal: vec3<f32>,
            @location(4) v_Position: vec3<f32>
        //!endif
    //!else
        @location(3) v_Position: vec3<f32>
    //!endif
}

@vertex
fn vs_main(
    model: VertexInput,
    instance: InstanceInput,
) -> VertexOutput {
    let model_matrix = mat4x4<f32>(
        instance.model_matrix_0,
        instance.model_matrix_1,
        instance.model_matrix_2,
        instance.model_matrix_3,
    );
    let normal_matrix = mat3x3<f32>(
        instance.normal_matrix_0,
        instance.normal_matrix_1,
        instance.normal_matrix_2,
    );
    let world_position = model_matrix * vec4<f32>(model.position, 1.0);
    
    var out: VertexOutput;

    //!ifdef HAS_NORMALS
        //!ifdef HAS_TANGENTS
            let normalW = normalize((model_matrix * vec4<f32>(model.normal.xyz, 0.0)).xyz);
            let tangentW = normalize((model_matrix * vec4<f32>(model.tangent.xyz, 0.0)).xyz);
            let bitangentW = cross(normalW, tangentW) * model.tangent.w;
            out.v_TBN_0 = tangentW;
            out.v_TBN_1 = bitangentW;
            out.v_TBN_2 = normalW;
        //!else
            out.v_Normal = normalize((model_matrix * vec4<f32>(model.normal.xyz, 0.0)).xyz);
        //!endif
    //!endif


    //!ifdef HAS_UV
    out.v_UV_0 = model.tex_coord_0;
    out.v_UV_1 = model.tex_coord_1;
    //!else
    out.v_UV_0 = vec2<f32>(0.0, 0.0);
    out.v_UV_1 = vec2<f32>(0.0, 0.0);
    //!endif

    out.gl_Position = u_mpv_matrix * vec4<f32>(model.position, 1.0);
    out.v_Position = model.position;

    return out;
}

// Encapsulate the various inputs used by the various functions in the shading equation
// We store values in this struct to simplify the integration of alternative implementations
// of the shading terms, outlined in the Readme.MD Appendix.
struct PBRInfo {
    NdotL: f32,                     // cos angle between normal and light direction
    NdotV: f32,                     // cos angle between normal and view direction
    NdotH: f32,                     // cos angle between normal and half vector
    LdotH: f32,                     // cos angle between light direction and half vector
    VdotH: f32,                     // cos angle between view direction and half vector
    perceptualRoughness: f32,       // roughness value, as authored by the model creator (input to shader)
    metalness: f32,                 // metallic value at the surface
    reflectance0: vec3<f32>,        // full reflectance color (normal incidence angle)
    reflectance90: vec3<f32>,       // reflectance color at grazing angle
    alphaRoughness: f32,            // roughness mapped to a more linear change in the roughness (proposed by [2])
    diffuseColor: vec3<f32>,        // color contribution from diffuse lighting
    specularColor: vec3<f32>,       // color contribution from specular lighting
    v_Position: vec3<f32>,
    v_UV_0: vec2<f32>,
    //!ifdef HAS_NORMALS
    //!ifdef HAS_TANGENTS
    v_TBN: mat3x3<f32>,
    //!else
    v_Normal: vec3<f32>,
    //!endif
    //!endif
    v_UV_1: vec2<f32>
}

let M_PI: f32 = 3.141592653589793;
let c_MinRoughness: f32 = 0.04;

// Find the normal for this fragment, pulling either from a predefined normal map
// or from the interpolated mesh normal and tangent attributes.
fn getNormal(info: PBRInfo, ) -> vec3<f32> 
{
    //!ifndef HAS_TANGENTS
        let pos_dx = dpdx(info.v_Position);
        let pos_dy = dpdy(info.v_Position);
        let tex_dx = dpdx(vec3<f32>(info.v_UV_0, 0.0));
        let tex_dy = dpdy(vec3<f32>(info.v_UV_0, 0.0));

        // compared to glsl version:
        // xyzw
        // stpq
        // rgba
        // all are valid for some reason
        let t = (tex_dy.y * pos_dx - tex_dx.y * pos_dy) / (tex_dx.x * tex_dy.y - tex_dy.x * tex_dx.y);

        //!ifdef HAS_NORMALS
            let ng = normalize(info.v_Normal);
        //!else
            let ng = cross(pos_dx, pos_dy);
        //!endif

        let t = normalize(t - ng * dot(ng, t));

        let b = normalize(cross(ng, t));
        let tbn = mat3x3<f32>(t, b, ng);
    //!else
        let tbn = info.v_TBN;
    //!endif

    //!ifdef HAS_NORMALMAP 
        // TODO: replace constant v_UV with array
        let n = textureSample(normal_tex, normal_sampler, info.v_UV_0).xyz;
        let n = normalize(tbn * ((2.0 * n - 1.0) * vec3<f32>(material_normal_scale, material_normal_scale, 1.0)));
    //!else
        let n = normalize(tbn[2].xyz);
    //!endif

    // reverse backface normals
    // TODO!: correct/best place? -> https://github.com/KhronosGroup/glTF-WebGL-PBR/issues/51
    return n;
}

struct FragOut {
    @location(0) albedo: vec3<f32>,
    @location(1) normal: vec3<f32>,
    @location(2) 
}

@fragment
fn fs_main(in: VertexOutput) -> @location(0) vec4<f32> {
    // Metallic and Roughness material properties are packed together
    // In glTF, these factors can be specified by fixed scalar values
    // or from a metallic-roughness map
    var perceptualRoughness = material_roughness_factor;
    var metallic = material_metallic_factor;

    //!ifdef HAS_METALROUGHNESSMAP
        // Roughness is stored in the 'g' channel, metallic is stored in the 'b' channel.
        // This layout intentionally reserves the 'r' channel for (optional) occlusion map data
        let mrSample = textureSample(metallic_roughness_tex, metallic_roughness_sampler, in.v_UV_0);
        perceptualRoughness = mrSample.g * perceptualRoughness;
        metallic = mrSample.b * metallic;
    //!endif

    perceptualRoughness = clamp(perceptualRoughness, c_MinRoughness, 1.0);
    metallic = clamp(metallic, 0.0, 1.0);
    // Roughness is authored as perceptual roughness; as is convention,
    // convert to material roughness by squaring the perceptual roughness [2].
    let alphaRoughness = perceptualRoughness * perceptualRoughness;
    //!ifdef HAS_BASECOLORMAP
        let base_color = textureSample(base_color_tex, base_color_sampler, in.v_UV_0) * material_base_color_factor;
    //!else
        let base_color = material_base_color_factor;
    //!endif

    let f0 = vec3<f32>(0.04);


    let specular_color = mix(f0, base_color.rgb, metallic);

    let refelctance = max(max(specular_color.r, specular_color.g), specular_color.b);

    // For typical incident reflectance range (between 4% to 100%) set the grazing reflectance to 100% for typical fresnel effect.
    // For very low reflectance range on highly diffuse objects (below 4%), incrementally reduce grazing reflecance to 0%.
    let reflectance90 = clamp(refelctance * 25.0, 0.0, 1.0);
    let specular_environmentR0 = specular_color.rgb;
    let specular_environmentR90 = vec3<f32>(1.0) * reflectance90;

    var pbr_info: PBRInfo;

    pbr_info.v_Position = in.gl_Position.xyz;
    pbr_info.v_UV_0 = in.v_UV_0;
    pbr_info.v_UV_1 = in.v_UV_1;
    //!ifdef HAS_NORMALS
        //!ifdef HAS_TANGENTS
            pbr_info.v_TBN = mat3x3<f32>(
                in.v_TBN_0,
                in.v_TBN_1,
                in.v_TBN_2,
            );
        //!else
            pbr_info.v_Normal = in.v_Normal;
        //!endif
    //!endif


    let n = getNormal(pbr_info);

    //!ifdef HAS_OCCLUSIONMAP
        let ao = textureSample(occlusion_tex, occlusion_sampler, in.v_UV_0).r;
        let color = mix(color, color * ao, material_occlusion_strength);
    //!endif

    //!ifdef HAS_EMISSIVEMAP
        let emissive = textureSample(emissive_tex, emissive_sampler, in.v_UV_0).rgb * material_emissive_factor;
        let color = color + emissive;
    //!endif

    var alpha = mix(1.0, base_color.a, u_alpha_blend);
    if u_alpha_cutoff > 0.0 {
        alpha = step(u_alpha_cutoff, base_color.a);
    }

    if alpha == 0.0 {
        discard;
    }
    

    return vec4<f32>(color, alpha);
    // return mrSample;
}
