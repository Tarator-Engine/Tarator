use tar_types::{components::Info, EngineState};

pub fn entity_list(ui: &mut egui::Ui, world: &mut tar_ecs::world::World, state: &mut EngineState) {
    egui::ScrollArea::vertical().show(ui, |ui| {
        let entitys = world.component_query::<Info>();
        for info in entitys {
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
}

pub fn complete(ui: &mut egui::Ui, world: &mut tar_ecs::world::World, state: &mut EngineState) {
    entity_list(ui, world, state);
    component_viewer(ui, world, state);
}
