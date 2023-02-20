use std::time::Duration;

use egui::ClippedPrimitive;

#[derive(Clone)]
pub struct EngineState {
    pub dt: Duration,
    pub fps: u32,
    pub size: winit::dpi::PhysicalSize<u32>,
    pub halt: bool,
    #[allow(unused)]
    pub view_rect: winit::dpi::PhysicalSize<u32>,
    pub cam_sensitivity: f32,
    pub paint_jobs: Vec<ClippedPrimitive>,
    pub egui_textures_delta: egui::epaint::textures::TexturesDelta,
    pub events: Vec<winit::event::WindowEvent<'static>>,
    pub mouse_movement: (f64, f64),
}
