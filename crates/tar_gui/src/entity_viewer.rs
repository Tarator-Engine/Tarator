use tar_types::{components::Info, EngineState};

pub fn entity_list(ui: &mut egui::Ui, world: &mut tar_ecs::world::World, state: &mut EngineState) {
    egui::ScrollArea::vertical().show(ui, |ui| {
        let entitys = world.entity_query::<Info>();
        for entity in entitys {
            let info = world.entity_get::<Info>(entity).unwrap();
            ui.with_layout(ui.layout().with_cross_justify(true), |ui| {
                let clicked = ui.selectable_label(false, info.name.clone()).clicked();

                if clicked {
                    // state.active_entity = entity;
                }
            });
        }
    });
}

pub fn component_viewer(
    ui: &mut egui::Ui,
    world: &mut tar_ecs::world::World,
    state: &mut EngineState,
) {
    let components: Vec<Box<dyn GuiEntity>> = world.get_entity_components_mut(state.active_entity);
}

pub fn complete(ui: &mut egui::Ui, world: &mut tar_ecs::world::World, state: &mut EngineState) {
    entity_list(ui, world, state);
    component_viewer(ui, world, state);
}
