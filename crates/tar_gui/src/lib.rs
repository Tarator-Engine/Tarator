#[derive(Default)]
pub struct GuiInData {
    pub dt: std::time::Duration,
    pub fps: u32,
    pub game_view_texture: Option<egui::TextureHandle>,
}

#[derive(Default)]
pub struct GuiOutData {
    pub mouse_in_game_view: bool,
}

pub fn gui(context: &egui::Context, state: &mut GuiInData) -> GuiOutData {
    let mut out = GuiOutData::default();
    egui::Window::new("Timings")
        .resizable(false)
        .show(context, |ui| {
            ui.label("Here you can see different frame timings");
            ui.label(format!("Frame time: {:?}", state.dt));
            ui.label(format!("FPS: {}", state.fps));
        });

    egui::CentralPanel::default()
        .frame(egui::Frame::default().fill(egui::Color32::TRANSPARENT))
        .show(&context, |ui| {
            out.mouse_in_game_view = ui.ui_contains_pointer()
        });

    out
}
