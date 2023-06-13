use std::{fs, path::PathBuf};

use egui::{ScrollArea, SidePanel, TopBottomPanel, Ui};

/// This data is preserved across frames
#[derive(Default)]
pub struct GuiInData {
    pub dt: std::time::Duration,
    pub fps: u32,
    pub game_view_texture: Option<egui::TextureHandle>,
    pub running: bool,
    pub file_system_gui: FilesystemGui,
}

/// This data is generated each frame in the gui function
#[derive(Default)]
pub struct GuiOutData {
    pub mouse_in_game_view: bool,
    pub reload_scripts: bool,

    pub load_model: Option<String>,
}

pub fn gui(context: &egui::Context, state: &mut GuiInData) -> GuiOutData {
    let mut out = GuiOutData::default();

    SidePanel::left("side_panel")
        .resizable(true)
        .show(context, |ui| {
            ui.label("Here you can see different frame timings:");
            ui.label(format!("Frame time: {:?}", state.dt));
            ui.label(format!("FPS: {}", state.fps));
        });

    egui::TopBottomPanel::top(egui::Id::new("top_panel")).show(context, |ui| {
        ui.vertical_centered_justified(|ui| {
            egui::Grid::new("buttons-grid").show(ui, |ui| {
                if !state.running && ui.button("▶").clicked() {
                    state.running = true;
                };
                if state.running && ui.button("⏸").clicked() {
                    state.running = false;
                }
                out.reload_scripts = ui.button("⟲").clicked();
            })
        });
    });

    egui::CentralPanel::default()
        .frame(egui::Frame::default().fill(egui::Color32::TRANSPARENT))
        .show(context, |ui| {
            out.mouse_in_game_view = ui.ui_contains_pointer()
        });

    out.load_model = state.file_system_gui.ui(context);

    egui::CentralPanel::default()
        .frame(egui::Frame::default().fill(egui::Color32::TRANSPARENT))
        .show(context, |ui| {
            out.mouse_in_game_view = ui.ui_contains_pointer()
        });

    out
}

pub enum FileAction {
    None,
    Rename,
    CreateFile,
    CreateDirectory,
}

pub struct FilesystemGui {
    current_directory: PathBuf,
    selected_path: Option<PathBuf>,
    file_action: FileAction,
    input_text: String,
}

impl Default for FilesystemGui {
    fn default() -> Self {
        FilesystemGui {
            current_directory: std::env::current_dir().unwrap(),
            selected_path: None,
            file_action: FileAction::None,
            input_text: String::new(),
        }
    }
}

impl FilesystemGui {
    fn list_directory_contents(&self) -> (Vec<PathBuf>, Vec<PathBuf>) {
        let mut directories = vec![];
        let mut files = vec![];

        if let Ok(dir) = fs::read_dir(&self.current_directory) {
            for entry in dir {
                if let Ok(entry) = entry {
                    if entry.path().is_dir() {
                        directories.push(entry.path());
                    } else {
                        files.push(entry.path());
                    }
                }
            }
        }
        (directories, files)
    }

    fn create_directory(&self, name: &str) -> Result<(), std::io::Error> {
        let new_dir = self.current_directory.join(name);
        fs::create_dir(new_dir)
    }

    fn create_file(&self, name: &str) -> Result<(), std::io::Error> {
        let new_file = self.current_directory.join(name);
        fs::File::create(new_file).map(|_| ())
    }

    fn rename(&self, old_name: &str, new_name: &str) -> Result<(), std::io::Error> {
        let old_path = self.current_directory.join(old_name);
        let new_path = self.current_directory.join(new_name);
        fs::rename(old_path, new_path)
    }
    fn delete(&self, name: &str) -> Result<(), std::io::Error> {
        let path = self.current_directory.join(name);
        if path.is_dir() {
            fs::remove_dir_all(path)
        } else {
            fs::remove_file(path)
        }
    }
}

impl FilesystemGui {
    fn ui(&mut self, ctx: &egui::Context) -> Option<String> {
        let mut out = None;

        TopBottomPanel::bottom("bottom_panel").show(ctx, |tbp_ui| {
            let (_, files) = self.list_directory_contents();
            tbp_ui.label("Files");

            tbp_ui.columns(2, |uis| {
                let ui0 = &mut uis[0];
                ScrollArea::vertical()
                    .id_source("files")
                    .show(ui0, |saf_ui| {
                        for entry in &files {
                            // use &files to borrow instead of move
                            saf_ui.horizontal(|ui| {
                                ui.label("📄"); // File icon
                                if ui
                                    .button(entry.file_name().unwrap().to_string_lossy())
                                    .clicked()
                                {
                                    self.selected_path = Some(entry.clone()); // need to clone here as we only borrowed entry
                                }
                            });
                        }
                    });
                let ui1 = &mut uis[1];

                ui1.vertical(|ui| {
                    ui.label("Directories");
                    if ui.button("Go Up").clicked() {
                        if let Some(parent_dir) = self.current_directory.parent() {
                            self.current_directory = parent_dir.to_path_buf();
                        }
                    }
                });
                let (directories, _) = self.list_directory_contents();
                ScrollArea::vertical()
                    .id_source("directories")
                    .show(ui1, |ui| {
                        for entry in &directories {
                            // use &directories to borrow instead of move
                            ui.horizontal(|ui| {
                                ui.label("🐺"); // Folder icon
                                if ui
                                    .button(entry.file_name().unwrap().to_string_lossy())
                                    .clicked()
                                {
                                    self.current_directory = entry.clone(); // need to clone here as we only borrowed entry
                                }
                            });
                        }
                    });
            });

            tbp_ui.columns(2, |uis| {
                let ui0 = &mut uis[0];
                if let Some(path) = &self.selected_path {
                    ui0.label(format!("Selected file: {:?}", path));
                } else {
                    ui0.label("No file selected.");
                }

                if let Some(file_name) = &self.selected_path {
                    let file_name_str =
                        file_name.file_name().unwrap().to_string_lossy().to_string();
                    if file_name_str.ends_with(".gltf") || file_name_str.ends_with(".tarasset") {
                        if ui0.button("Load File").clicked() {
                            out = Some(file_name.to_string_lossy().to_string());
                        }
                    }
                }

                let ui1 = &mut uis[1];
                ui1.label(format!("Selected path: {:?}", self.current_directory));
            });

            self.file_management_ui(tbp_ui);
        });
        return out;
    }

    fn file_management_ui(&mut self, ui: &mut Ui) {
        ui.horizontal(|ui| {
            ui.label("Create:");
            if ui.button("Directory").clicked() {
                self.file_action = FileAction::CreateDirectory;
            }
            if ui.button("File").clicked() {
                self.file_action = FileAction::CreateFile;
            }
        });

        ui.horizontal(|ui| {
            ui.label("Rename:");
            if ui.button("Rename").clicked() {
                self.file_action = FileAction::Rename;
            }
        });

        ui.horizontal(|ui| {
            ui.label("Delete:");
            if ui.button("Delete").clicked() {
                if let Some(selected_path) = &self.selected_path {
                    let name = selected_path
                        .file_name()
                        .unwrap()
                        .to_string_lossy()
                        .to_string();
                    let _ = self.delete(&name);
                }
            }
        });

        match self.file_action {
            FileAction::Rename | FileAction::CreateFile | FileAction::CreateDirectory => {
                ui.horizontal(|ui| {
                    ui.label(match self.file_action {
                        FileAction::Rename => "New name:",
                        FileAction::CreateFile => "File name:",
                        FileAction::CreateDirectory => "Directory name:",
                        _ => "",
                    });
                    ui.text_edit_singleline(&mut self.input_text);
                    if ui.button("Apply").clicked() {
                        match self.file_action {
                            FileAction::Rename => {
                                if let Some(selected_path) = &self.selected_path {
                                    let old_name = selected_path
                                        .file_name()
                                        .unwrap()
                                        .to_string_lossy()
                                        .to_string();
                                    let _ = self.rename(&old_name, &self.input_text);
                                }
                            }
                            FileAction::CreateFile => {
                                let _ = self.create_file(&self.input_text);
                            }
                            FileAction::CreateDirectory => {
                                let _ = self.create_directory(&self.input_text);
                            }
                            _ => (),
                        }
                        self.file_action = FileAction::None;
                        self.input_text.clear();
                    }
                });
            }
            _ => (),
        }
    }
}
