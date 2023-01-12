use std::{sync::Arc, time::Instant};

use instant::Duration;
use parking_lot::RwLock;
use tar_render::render::forward::ForwardRenderer;

pub struct EngineState {
    game_renderer: RwLock<ForwardRenderer>,
    start_time: Instant,
    dt: Duration,
}
