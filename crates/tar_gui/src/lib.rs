pub fn gui(context: &egui::Context, state: &mut tar_types::EngineState) {
    egui::Window::new("Timings")
        .resizable(false)
        .show(&context, |ui| {
            ui.label("Here you can see different frame timings");
            ui.label(format!("Frame time: {:?}", state.dt));
            ui.label(format!("FPS: {}", state.fps));
        });
    egui::SidePanel::right("right panel")
        .resizable(true)
        .default_width(300.0)
        .show(&context, |ui| {
            ui.vertical_centered(|ui| ui.heading("right panel"))
        });
    egui::SidePanel::left("left panel")
        .resizable(true)
        .default_width(300.0)
        .show(&context, |ui| {
            ui.vertical_centered(|ui| ui.heading("left panel"));
            ui.label("sensitvity");
            ui.add(egui::Slider::new(&mut state.cam_sensitivity, 0.0..=5.0));
        });
    egui::TopBottomPanel::bottom("bottom panel")
        .resizable(true)
        .default_height(200.0)
        .show(&context, |ui| {
            ui.vertical_centered(|ui| ui.heading("bottom panel"))
        });
    egui::TopBottomPanel::top("top panel")
        .resizable(false)
        .default_height(50.0)
        .show(&context, |ui| {
            ui.vertical_centered(|ui| ui.heading("controls"))
        });
}
