use std::{mem::size_of, num::NonZeroU32, sync::Arc};

use wgpu::util::DeviceExt;

use crate::{
    material::{BindGroup, PbrMaterial, PerFrameData, PerFrameUniforms, PerMaterialUniforms},
    mesh::Mesh,
    node::Node,
    object::Object,
    primitive::{Instance, Primitive},
    shader::{MaterialInput, PbrShader},
    store::{store_node::StoreNode, store_object::StoreObject, store_primitive::StorePrimitive},
    texture::Texture,
    vertex::Vertex,
    Error, Result, WgpuInfo,
};

pub fn build_loaded(obj: StoreObject, w_info: Arc<WgpuInfo>) -> Result<Object> {
    let timer = tar_utils::start_timer_msg("started building loaded object");
    let obj = Arc::new(obj);

    let mut nodes = vec![];
    for node in &obj.nodes {
        if node.root_node {
            nodes.push(build_node(node, obj.clone(), w_info.clone())?)
        }
    }

    tar_utils::log_timing("loaded object in ", timer);

    Ok(Object { nodes })
}

pub fn build(source: String, w_info: Arc<WgpuInfo>) -> Result<Object> {
    let timer = tar_utils::start_timer_msg("started building object");
    let object: StoreObject = rmp_serde::from_slice(&std::fs::read(source)?)?;
    tar_utils::log_timing("loaded from disk in ", timer);

    build_loaded(object, w_info)
}

fn build_node(node: &StoreNode, object: Arc<StoreObject>, w_info: Arc<WgpuInfo>) -> Result<Node> {
    let timer = tar_utils::start_timer_msg("started building node");
    let mut children = vec![];
    let child_ids = &node.children;
    for id in child_ids {
        children.push(build_node(
            object
                .clone()
                .nodes
                .iter()
                .find(|n| (*n).index == *id)
                .ok_or(Error::NonexistentNode)?,
            object.clone(),
            w_info.clone(),
        )?);
    }

    let mesh = build_mesh(&node.mesh, object, w_info)?;

    tar_utils::log_timing("loaded node in ", timer);

    Ok(Node {
        index: node.index,
        children,
        mesh,
        rotation: node.rotation,
        scale: node.scale,
        translation: node.translation,
        name: node.name.clone(),
        final_transform: node.final_transform,
    })
}

fn build_mesh(
    mesh: &Option<usize>,
    object: Arc<StoreObject>,
    w_info: Arc<WgpuInfo>,
) -> Result<Option<Mesh>> {
    if let Some(id) = mesh {
        let timer = tar_utils::start_timer_msg("started building timer");
        let mesh = object
            .meshes
            .iter()
            .find(|m| (*m).index == *id)
            .ok_or(Error::NonexistentMesh)?;

        let prims = &mesh.primitives;
        let mut primitives = vec![];

        for prim in prims {
            primitives.push(build_primitive(prim, object.clone(), w_info.clone())?);
        }

        tar_utils::log_timing("loaded mesh in ", timer);

        Ok(Some(Mesh {
            index: mesh.index,
            name: mesh.name.clone(),
            primitives,
        }))
    } else {
        Ok(None)
    }
}

fn build_primitive(
    prim: &StorePrimitive,
    object: Arc<StoreObject>,
    w_info: Arc<WgpuInfo>,
) -> Result<Primitive> {
    let timer = tar_utils::start_timer_msg("started building primitive");
    let num_indices = prim.indices.as_ref().map(|i| i.len()).unwrap_or(0) as u32;
    let num_vertices = prim.vertices.len() as u32;

    let vertices = w_info
        .device
        .create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("vertex buffer"),
            contents: bytemuck::cast_slice(&prim.vertices),
            usage: wgpu::BufferUsages::VERTEX,
        });

    if prim.indices.is_none() {
        return Err(Error::NotSupported("models without indicies".into()));
    };
    let indices = w_info
        .device
        .create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("index buffer"),
            contents: bytemuck::cast_slice(prim.indices.as_ref().unwrap()),
            usage: wgpu::BufferUsages::INDEX,
        });

    let instance_data = vec![Instance {
        model: [[0.0; 4]; 4],
        normal: [[0.0; 3]; 3],
    }];
    let num_instances = instance_data.len() as u32;

    let instances = w_info
        .device
        .create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("Instance Buffer"),
            contents: bytemuck::cast_slice(&instance_data),
            usage: wgpu::BufferUsages::VERTEX,
        });
    let material = build_material(prim.material, object, w_info)?;

    tar_utils::log_timing("loaded primitive in ", timer);

    Ok(Primitive {
        num_vertices,
        vertices,
        num_indices,
        indices,
        instances,
        num_instances,
        material,
    })
}

fn build_material(
    id: usize,
    object: Arc<StoreObject>,
    w_info: Arc<WgpuInfo>,
) -> Result<PbrMaterial> {
    let timer = tar_utils::start_timer_msg("started building material");
    let mat = object
        .materials
        .iter()
        .find(|m| m.index == id)
        .ok_or(Error::NonExistentMaterial)?;

    let base_obj = object.clone();
    let info_clone = w_info.clone();
    let base_tex = mat.base_color_texture.clone();

    let base_color_thread =
        std::thread::spawn(move || build_texture(base_tex, base_obj, info_clone));

    let obj_clone = object.clone();
    let info_clone = w_info.clone();
    let met_rou_tex = mat.metallic_roughness_texture.clone();

    let metallic_roughness_thread =
        std::thread::spawn(move || build_texture(met_rou_tex, obj_clone, info_clone));

    let obj_clone = object.clone();
    let info_clone = w_info.clone();
    let normal_tex = mat.normal_texture.clone();

    let normal_thread =
        std::thread::spawn(move || build_texture(normal_tex, obj_clone, info_clone));

    let obj_clone = object.clone();
    let info_clone = w_info.clone();
    let occ_tex = mat.occlusion_texture.clone();

    let occlusion_thread =
        std::thread::spawn(move || build_texture(occ_tex, obj_clone, info_clone));

    let obj_clone = object.clone();
    let info_clone = w_info.clone();
    let emissive_tex = mat.emissive_texture.clone();

    let emissive_thread =
        std::thread::spawn(move || build_texture(emissive_tex, obj_clone, info_clone));

    let base_color_texture = base_color_thread.join().map_err(|_| Error::ThreadError)??;
    let metallic_roughness_texture = metallic_roughness_thread
        .join()
        .map_err(|_| Error::ThreadError)??;
    let normal_texture = normal_thread.join().map_err(|_| Error::ThreadError)??;
    let occlusion_texture = occlusion_thread.join().map_err(|_| Error::ThreadError)??;
    let emissive_texture = emissive_thread.join().map_err(|_| Error::ThreadError)??;

    let mut per_material_uniforms = PerMaterialUniforms {
        base_color_texture,
        metallic_roughness_texture,
        normal_texture,
        occlusion_texture,
        emissive_texture,
        bind_group: None,
    };

    let mut shader_flags = mat.shader_flags;

    shader_flags |= PbrMaterial::shader_flags(
        mat.base_color_texture.is_some(),
        mat.normal_texture.is_some(),
        mat.emissive_texture.is_some(),
        mat.metallic_roughness_texture.is_some(),
        mat.occlusion_texture.is_some(),
    );

    let per_frame = (
        PerFrameUniforms::bind_group_layout(),
        PerFrameUniforms::names(),
    );

    let per_material_entries = per_material_uniforms.entries();
    let per_material_bind_group_layouts =
        PerMaterialUniforms::bind_group_layout(&per_material_entries);
    let per_material = (
        per_material_bind_group_layouts.clone(),
        per_material_uniforms.names(),
    );

    let pbr_shader = PbrShader::new(
        shader_flags,
        MaterialInput {
            base_color_factor: mat.base_color_factor.into(),
            metallic_factor: mat.metallic_factor,
            roughness_factor: mat.roughness_factor,
            normal_scale: mat.normal_scale.unwrap_or(1.0),
            occlusion_strength: mat.occlusion_strength,
            emissive_factor: mat.emissive_factor.into(),
            alpha_cutoff: mat.alpha_cutoff.unwrap_or(1.0),
        },
        &[per_frame, per_material],
        w_info.clone(),
    )?;

    let per_frame_bind_group = w_info
        .clone()
        .device
        .create_bind_group_layout(&PerFrameUniforms::bind_group_layout());

    let per_material_bind_group = w_info
        .device
        .create_bind_group_layout(&per_material_bind_group_layouts);

    per_material_uniforms.set_bind_group(&w_info.device, &per_material_bind_group);

    let per_frame_uniforms = PerFrameUniforms::new(
        PerFrameData::default(),
        &per_frame_bind_group,
        w_info.clone(),
    );

    let pipeline_layout = w_info
        .device
        .create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
            label: Some("Material pipeline layout"),
            bind_group_layouts: &[&per_frame_bind_group, &per_material_bind_group],
            push_constant_ranges: &[],
        });

    let pipeline = w_info
        .device
        .create_render_pipeline(&wgpu::RenderPipelineDescriptor {
            label: Some(&format!("{:?}", pbr_shader.shader.module)),
            layout: Some(&pipeline_layout),
            vertex: wgpu::VertexState {
                module: &pbr_shader.shader.module,
                entry_point: "vs_main",
                buffers: &[Vertex::desc(), Instance::desc()],
            },
            fragment: Some(wgpu::FragmentState {
                module: &pbr_shader.shader.module,
                entry_point: "fs_main",
                targets: &[Some(wgpu::ColorTargetState {
                    format: w_info.surface_format,
                    blend: Some(wgpu::BlendState {
                        alpha: wgpu::BlendComponent::REPLACE,
                        color: wgpu::BlendComponent::REPLACE,
                    }),
                    write_mask: wgpu::ColorWrites::ALL,
                })],
            }),
            primitive: wgpu::PrimitiveState {
                topology: wgpu::PrimitiveTopology::TriangleList,
                strip_index_format: None,
                front_face: wgpu::FrontFace::Ccw,
                cull_mode: Some(wgpu::Face::Back),
                polygon_mode: wgpu::PolygonMode::Fill,
                unclipped_depth: false,
                conservative: false,
            },
            depth_stencil: Some(wgpu::DepthStencilState {
                format: wgpu::TextureFormat::Depth32Float,
                depth_compare: wgpu::CompareFunction::Less,
                stencil: wgpu::StencilState::default(),
                bias: wgpu::DepthBiasState::default(),
                depth_write_enabled: true,
            }),
            multisample: wgpu::MultisampleState {
                count: 1,
                mask: !0,
                alpha_to_coverage_enabled: false,
            },
            // If the pipeline will be used with a multiview render pass, this
            // indicates how many array layers the attachments will have.
            multiview: None,
        });

    tar_utils::log_timing("loaded material in ", timer);

    Ok(PbrMaterial {
        index: mat.index,
        name: mat.name.clone(),
        base_color_factor: mat.base_color_factor,
        metallic_factor: mat.metallic_factor,
        roughness_factor: mat.roughness_factor,
        normal_scale: mat.normal_scale,
        occlusion_strength: mat.occlusion_strength,
        emissive_factor: mat.emissive_factor,
        alpha_cutoff: mat.alpha_cutoff,
        alpha_mode: mat.alpha_mode.into(),
        double_sided: mat.double_sided,
        pbr_shader,
        per_frame_uniforms,
        per_material_uniforms,
        pipeline,
    })
}

fn build_texture(
    id: Option<usize>,
    object: Arc<StoreObject>,
    w_info: Arc<WgpuInfo>,
) -> Result<Option<Texture>> {
    let timer = tar_utils::start_timer_msg("started building texture");
    if id.is_none() {
        return Ok(None);
    };
    let id = id.unwrap();
    let tex = object
        .textures
        .iter()
        .find(|t| t.index == id)
        .ok_or(Error::NonExistentTexture)?;

    let img = image::open(&tex.path)?;

    let dims = (img.width(), img.height());

    use image::DynamicImage::*;
    let (format, data_layout, data): (_, _, Vec<u8>) = match &img {
        ImageLuma8(d) => (
            wgpu::TextureFormat::R8Unorm, // TODO: confirm if these are correct
            wgpu::ImageDataLayout {
                offset: 0,
                bytes_per_row: NonZeroU32::new(dims.0 * size_of::<image::Luma<u8>>() as u32),
                rows_per_image: NonZeroU32::new(dims.1),
            },
            d.to_vec(),
        ),
        ImageLumaA8(d) => (
            wgpu::TextureFormat::Rg8Unorm,
            wgpu::ImageDataLayout {
                offset: 0,
                bytes_per_row: NonZeroU32::new(dims.0 * size_of::<image::LumaA<u8>>() as u32),
                rows_per_image: NonZeroU32::new(dims.1),
            },
            d.to_vec(),
        ),
        ImageRgb8(_) => {
            let d = img.to_rgba8();
            (
                wgpu::TextureFormat::Rgba8Unorm,
                wgpu::ImageDataLayout {
                    offset: 0,
                    bytes_per_row: NonZeroU32::new(dims.0 * size_of::<image::Rgba<u8>>() as u32),
                    rows_per_image: NonZeroU32::new(dims.1),
                },
                d.to_vec(),
            )
        }
        ImageRgba8(d) => (
            wgpu::TextureFormat::Rgba8Unorm,
            wgpu::ImageDataLayout {
                offset: 0,
                bytes_per_row: NonZeroU32::new(dims.0 * size_of::<image::Rgba<u8>>() as u32),
                rows_per_image: NonZeroU32::new(dims.1),
            },
            d.to_vec(),
        ),
        _ => {
            return Err(Error::NotSupported(
                "image formats with pixels that are not 8 bit".to_owned(),
            ))
        }
    };

    let size = wgpu::Extent3d {
        width: dims.0,
        height: dims.1,
        depth_or_array_layers: 1,
    };

    let texture = w_info.device.create_texture(&wgpu::TextureDescriptor {
        label: Some("texture"),
        size,
        mip_level_count: 1,
        sample_count: 1,
        dimension: wgpu::TextureDimension::D2,
        format,
        usage: wgpu::TextureUsages::all(),
    });

    w_info.queue.write_texture(
        wgpu::ImageCopyTexture {
            aspect: wgpu::TextureAspect::All,
            texture: &texture,
            mip_level: 0,
            origin: wgpu::Origin3d::ZERO,
        },
        &data,
        data_layout,
        size,
    );

    let view = texture.create_view(&wgpu::TextureViewDescriptor::default());
    let sampler = w_info.device.create_sampler(&wgpu::SamplerDescriptor {
        address_mode_u: wgpu::AddressMode::ClampToEdge,
        address_mode_v: wgpu::AddressMode::ClampToEdge,
        address_mode_w: wgpu::AddressMode::ClampToEdge,
        mag_filter: wgpu::FilterMode::Linear,
        min_filter: wgpu::FilterMode::Nearest,
        mipmap_filter: wgpu::FilterMode::Nearest,
        ..Default::default()
    });

    tar_utils::log_timing("loaded texture in ", timer);

    Ok(Some(Texture {
        index: tex.index,
        name: None,
        texture,
        view,
        sampler,
        tex_coord: 0,
    }))
}
