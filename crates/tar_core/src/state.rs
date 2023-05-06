use egui::{ClippedPrimitive, TexturesDelta};

#[derive(Debug, Clone, Default)]
pub struct ShareState {
    pub halt: bool,

    pub window_size: winit::dpi::PhysicalSize<u32>,

    pub dt: std::time::Duration,

    pub device_events: Vec<winit::event::DeviceEvent>,

    pub paint_jobs: Vec<ClippedPrimitive>,
    pub egui_textures_delta: TexturesDelta,
    pub resize: bool,
    pub mouse_pressed: bool,
    pub mouse_in_view: bool,
}
#[derive(Debug)]
pub struct MainThreadState {
    pub window: winit::window::Window,
    pub scripts_lib: Option<libloading::Library>,
    pub scripts_systems: Option<scr_types::Systems>,
}
