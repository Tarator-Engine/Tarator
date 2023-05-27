use tar_res::SomeResult;

pub fn add_gltf_object(path: String) -> tar_res::SomeResult<Vec<String>> {
    let models = tar_res::import_models(&path)?;

    let paths: Vec<SomeResult<String>> = models
        .iter()
        .map(|model| -> tar_res::SomeResult<String> {
            let path = format!("res/{}.tarasset", model.name);
            std::fs::write(&path, ron::to_string(&model)?)?;
            Ok(path)
        })
        .collect();

    let mut fin_paths = vec![];

    for path in paths {
        fin_paths.push(path?);
    }

    Ok(fin_paths)
}

pub fn load_object_to_renderer(
    render_state: &mut tar_render::state::RenderState,
    path: String,
    id: uuid::Uuid,
) -> tar_res::SomeResult<()> {
    let model: tar_res::model::Model = ron::from_str(&std::fs::read_to_string(path)?)?;

    let loaded_model = tar_render::model::Model::from_stored(
        model,
        &render_state.device,
        &render_state.queue,
        render_state.config.format,
    );

    render_state.models.insert(id, loaded_model);

    Ok(())
}
