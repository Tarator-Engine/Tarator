use crate::prelude::*;

#[test]
fn visualizer() {
    let session = Session::new("./target/visualizer.json");
    _500();
    _500_200_100();
    _50_and_600_240_90();
    _600_240_90();
    _500_and_50_and_600_240_90_and_600_240_90_and_420();
    session.end();
}

#[trace]
fn _500() {
    pause_for(500);
}

#[trace]
fn _500_200_100() {
    pause_for(500);
    pause_for(200);
    pause_for(100);
    
}

#[trace]
fn _50_and_600_240_90() {
    pause_for(50);
    _600_240_90();
}

#[trace]
fn _600_240_90() {
    pause_for(600);
    pause_for(240);
    pause_for(90);
}

#[trace]
fn _500_and_50_and_600_240_90_and_600_240_90_and_420() {
    _500();
    _50_and_600_240_90();
    _600_240_90();
    pause_for(420);
}

#[trace]
fn pause_for(micros: u64) {
    std::thread::sleep(std::time::Duration::from_micros(micros));
}

