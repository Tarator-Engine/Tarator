pub struct GUI {
    renderer: tar_render::EditorRenderer,
}


impl GUI {
    pub fn new<'a>(cc: &'a eframe::CreationContext<'a>) -> Self {
        Self {
            renderer: tar_render::EditorRenderer::new(cc).unwrap(),
        }
    }
}

impl eframe::App for GUI {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        egui::CentralPanel::default().show(ctx, |ui| {
            egui::ScrollArea::both()
                .auto_shrink([false; 2])
                .show(ui, |ui| {
                    ui.horizontal(|ui| {
                        ui.spacing_mut().item_spacing.x = 0.0;
                        ui.label("The scene is being painted using ");
                        ui.hyperlink_to("WGPU", "https://wgpu.rs");
                        ui.label(" (Portable Rust graphics API awesomeness)");
                    });
                    ui.label("It's not a very impressive demo, but it shows you can embed 3D inside of egui.");
                    let _ = ui.button("text");

                    egui::Frame::canvas(ui.style()).show(ui, |ui| {
                        self.renderer.custom_painting(ui);
                    });
                    ui.label("Drag to rotate!");
                });
        });
    }
}

pub async fn run() {
    let options = eframe::NativeOptions {
        drag_and_drop_support: true,

        initial_window_size: Some([1280.0, 1024.0].into()),

        renderer: eframe::Renderer::Wgpu,

        ..Default::default()
    };
    eframe::run_native("My egui App", options, Box::new(|cc| Box::new(GUI::new(cc))));
}
