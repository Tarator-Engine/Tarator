use std::path::PathBuf;

fn main() {
    //tar_scripting::run();
    let mut v1 = vec![PathBuf::from("assets/test.rs"),PathBuf::from("assets/test2.rs")];
    tar_scripting::compile_files(&mut v1);
}
