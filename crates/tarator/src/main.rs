fn main() {
    // if cfg!(target_arch = "wasm32") {
    //     std::panic::set_hook(Box::new(console_error_panic_hook::hook));
    //     console_log::init_with_level(log::Level::Info).expect("Couldn't initialize logger");
    // } else {
    //     env_logger::init();
    // }
    env_logger::init();
    pollster::block_on(tar_core::run());

    // let (lib, systems) = tar_abi::load_scripts_lib().unwrap();
    // tar_abi::run_scripts(&lib, &systems);
}
