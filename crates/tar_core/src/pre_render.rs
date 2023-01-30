use std::sync::{Arc, Barrier};

use parking_lot::RwLock;

pub fn pre_render_fn(pre_render_s: Arc<RwLock<bool>>, p_barrier: Arc<Barrier>) {
    let mut frames = 0;
    let mut fps = 0;
    let mut since_start = 0;
    let start_time = instant::Instant::now();

    let mut view_rect = (800, 800);

    let mut last_render_time = start_time;

    loop {
        if let Some(_) = pre_render_s.read().then_some(true) {
            return;
        }

        // do pre_rendering here
        // scripts run

        // run physics

        p_barrier.wait();
    }
}
