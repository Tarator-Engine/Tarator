use crate::prelude::*;

#[test]
#[session("target/visualizer.json")]
fn visualizer() {
    assert!(_500() == "should compile");
    assert!(_500_200_100() == "should compile");
    _50_and_600_240_90();
    _600_240_90();
    _500_and_50_and_600_240_90_and_600_240_90_and_420();
}

#[test]
#[session("target/no_segfault.json")]
fn no_segfault() {

    let num = 1900;
    let _ = second_session(&num);
}

#[session("target/second")]
fn second_session<'a>(num: &'a u32) -> &'a u32 {
    assert!(_500() == "should compile");
    assert!(_500_200_100() == "should compile");
    _50_and_600_240_90();
    _600_240_90();
    _500_and_50_and_600_240_90_and_600_240_90_and_420();   
    num
}

#[trace]
fn _500<'a: 'static>() -> &'a str {
    pause_for(500);
    return "should compile";
}

#[trace]
fn _500_200_100<'a: 'static>() -> &'a str {
    pause_for(500);
    pause_for(200);
    pause_for(100);
    "should compile"
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

