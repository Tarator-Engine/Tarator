fn main() {
    pollster::block_on(tar_assets::reset_cache()).unwrap();
    let _id = pollster::block_on(tar_assets::import_gltf(std::path::PathBuf::from("/home/slackers/Documents/gltf-assets/helmet/FlightHelmet.gltf"), None)).unwrap();

}
