
fn main() {
    env_logger::init();
     pollster::block_on(tar_render::dev::run());
}
