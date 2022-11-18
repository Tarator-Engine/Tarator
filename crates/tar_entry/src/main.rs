fn main() {
    pollster::block_on(tar_assets::reset_cache()).unwrap();
    let _id = pollster::block_on(tar_assets::import_gltf(std::path::PathBuf::from("C:/Users/slackers/Desktop/BarramundiFish.glb"), None)).unwrap();

}
