use scr_types::prelude::GameState;

fn main() {
    // if cfg!(target_arch = "wasm32") {
    //     std::panic::set_hook(Box::new(console_error_panic_hook::hook));
    //     console_log::init_with_level(log::Level::Info).expect("Couldn't initialize logger");
    // } else {
    //     env_logger::init();
    // }
    env_logger::init();
    // pollster::block_on(tar_core::run());

    let (lib, systems) = tar_abi::load_scripts_lib().unwrap();
    let game_state = GameState {
        dt: std::time::Duration::from_millis(2),
    };
    tar_abi::run_scripts(&lib, &systems, &game_state);
    tar_abi::add_basic_model(&lib, uuid::Uuid::new_v4());
    let data = tar_abi::get_render_data(&lib);
    dbg!(&data);
    tar_abi::save_world(&lib);
    tar_abi::load_world(&lib);
}
