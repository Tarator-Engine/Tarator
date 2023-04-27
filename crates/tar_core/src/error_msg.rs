use std::error::Error;

pub fn error_message(ctx: &egui::Context, err: &Box<dyn Error>) -> bool {
    egui::Window::new("Error ")
        .resizable(true)
        .show(ctx, |ui| {
            ui.heading("An Error Occured:");
            ui.label(format!("{err}"));
            ui.button("Ok").clicked()
        })
        .unwrap()
        .inner
        .unwrap()
}
