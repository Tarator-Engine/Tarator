#[derive(Debug, Default)]
pub struct GuiData {
    dt: std::time::Duration,
    fps: u16,
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
