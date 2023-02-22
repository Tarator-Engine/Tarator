use std::time::Duration;

use egui::ClippedPrimitive;

#[allow(unused)]
use tar_ecs::world::World;

/// This struct stores all the important state of the engine.
///
/// it and a [`World`] is stored as state
#[derive(Clone)]
pub struct EngineState {
    pub dt: Duration,
    pub fps: u32,
    pub size: winit::dpi::PhysicalSize<u32>,
    pub halt: bool,
    pub view_rect: egui::Rect,
    pub cam_sensitivity: f32,
    pub paint_jobs: Vec<ClippedPrimitive>,
    pub egui_textures_delta: egui::epaint::textures::TexturesDelta,
    // pub events: Vec<winit::event::WindowEvent<'static>>,
    pub mouse_movement: (f64, f64),
    pub mouse_pos: egui::Pos2,
    pub mouse_in_view: bool,
    pub add_object_string: String,
    pub add_object: bool,
    pub adding_object: bool,
    pub editing: bool,
}

impl Default for EngineState {
    fn default() -> Self {
        Self {
            dt: Duration::from_secs(0),
            fps: 0,
            size: winit::dpi::PhysicalSize::new(0, 0),
            halt: false,
            view_rect: egui::Rect::NOTHING,
            cam_sensitivity: 0.4,
            paint_jobs: vec![],
            egui_textures_delta: egui::epaint::textures::TexturesDelta::default(),
            // events: vec![],
            mouse_movement: (0.0, 0.0),
            mouse_pos: egui::Pos2::ZERO,
            mouse_in_view: false,
            add_object: false,
            add_object_string: String::new(),
            adding_object: false,
            editing: false,
        }
    }
}
