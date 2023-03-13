#[tokio::main]
async fn main() {
    let mut s = tar_scripting::Scripting::default();
    s.load_scripts().unwrap()
}
