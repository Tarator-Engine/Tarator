use tar_render::model::Model;

pub fn add_gltf_object(
    render_state: &mut tar_render::state::RenderState,
    path: String,
) -> tar_res::SomeResult<Vec<uuid::Uuid>> {
    let models = tar_res::import_models(&path)?;

    let models: Vec<Model> = models
        .into_iter()
        .map(|model| {
            tar_render::model::Model::from_stored(
                model,
                &render_state.device,
                &render_state.queue,
                render_state.config.format,
            )
        })
        .collect();

    let mut ids = vec![];

    for model in models {
        let uid = uuid::Uuid::new_v4();
        ids.push(uid);
        render_state.models.insert(uid, model);
    }

    Ok(ids)
}
