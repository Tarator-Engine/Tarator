use scr_types::{components::Info};
use tar_types::EngineState;

pub fn entity_list(
    ui: &mut egui::Ui,
    state: &mut EngineState,
) {
    egui::ScrollArea::vertical().show(ui, |ui| {
        // world.component_query::<Info>(|_, info| {
        //     ui.with_layout(ui.layout().with_cross_justify(true), |ui| {
        //         let clicked = ui.selectable_label(false, info.name.clone()).clicked();
        //
        //         if clicked {
        //             // state.active_entity = entity;
        //         }
        //     });
        // })
    });
}

pub fn component_viewer(
    ui: &mut egui::Ui,
    state: &mut EngineState,
) {
}

pub fn complete(ui: &mut egui::Ui, state: &mut EngineState) {
    entity_list(ui, state);
    component_viewer(ui, state);
}
