#[derive(Default)]
pub struct GuiData {
    pub dt: std::time::Duration,
    pub fps: u32,
    pub game_view_texture: Option<egui::TextureHandle>,
}

pub fn gui(context: &egui::Context, state: &mut GuiData) {
    egui::Window::new("Timings")
        .resizable(false)
        .show(&context, |ui| {
            ui.label("Here you can see different frame timings");
            ui.label(format!("Frame time: {:?}", state.dt));
            ui.label(format!("FPS: {}", state.fps));
        });
}
