use std::{fs, io};
use libc::{c_void, dlclose, dlopen, dlsym, RTLD_NOW};
use std::ffi::CString;
use std::fs::File;
use std::io::prelude::*;
use std::io::SeekFrom;
use std::process::Command;
use std::error::Error;


pub struct Script {
    // start: Option<StatrFun>,
    // update: Option<UpdateFun>,
    // fixed_update: Option<FixedUpdateFun>,
}

pub fn compile_files(files: &mut Vec<std::path::PathBuf>) -> (){
        let mut jit = jit::JitEngine::new();
        for file in files.iter() {
            println!("{:?}", file);
            let file_content = fs::read_to_string(file)
                .expect("Should have been able to read the file");

            println!("{file_content}");
            let fun = jit.compile(&file_content);
            let result = fun.call(10, 20);
            println!("{:?}\n", result);
        }
}


mod jit {
    use std::ffi::{c_void, CStr, CString};
    use std::fs::File;
    use std::io;
    use std::io::{Seek, SeekFrom, Write};
    use std::process::Command;
    use libc::{dlclose, dlopen, dlsym, RTLD_NOW, user_regs_struct};

    const SOURCE_PATH: &'static str = "/tmp/jit.rs";
    const LIB_PATH: &'static str = "/tmp/librsjit.so";
    const FUN_NAME: &'static str = "calculate";

    // A JIT engine using rustc backed by a single source file.
    pub struct JitEngine {
        file: File,
    }

    impl JitEngine {
        pub fn new() -> Self {
            let file = File::create(SOURCE_PATH).expect("Could not create file");
            Self { file }
        }

        pub fn compile(&mut self, expression: &String) -> Fun {
            // Reset the source file
            self.file.set_len(0).unwrap();
            self.file.seek(SeekFrom::Start(0)).unwrap();

            // Write the rust program
            self.file
                .write_all(
                    expression
                        .as_bytes(),
                )
                .unwrap();

            // Compile the sources
            Command::new("rustc")
                .args(&["--crate-type=dylib", SOURCE_PATH, "-o"])
                .arg(LIB_PATH)
                .status()
                .unwrap();

            unsafe { Fun::new(LIB_PATH, FUN_NAME) }
        }
    }

    // A function from a library dynamically linked.
    pub struct Fun {
        fun: fn(a: i32, b: i32) -> i32,
        handle: *mut c_void,
    }

    impl Fun {
        unsafe fn new(lib_path: &str, fun_name: &str) -> Fun
        {
            // Load the library
            let filename = CString::new(lib_path).unwrap();
            // let handle = match handle {
            //     Ok(handle) => dlopen(filename.as_ptr(), RTLD_NOW),
            //     Err(e) => return Err(e)
            // }
            let handle = dlopen(filename.as_ptr(), RTLD_NOW);

            if handle.is_null() {
                panic!("Failed to resolve dlopen")
            }

            // Look for the function in the library
            let fun_name = CString::new(fun_name).unwrap();
            let fun = dlsym(handle, fun_name.as_ptr());
            if fun.is_null() {
                panic!("Failed to resolve '{}'", &fun_name.to_str().unwrap());
            }

            // dlsym returns a C 'void*', cast it to a function pointer
            let fun = std::mem::transmute::<*mut c_void, fn(i32, i32) -> i32>(fun);
            Self { fun, handle }
        }

        pub fn call(&self, a: i32, b: i32) -> i32 {
            (self.fun)(a, b)
        }
    }

    impl Drop for Fun {
        fn drop(&mut self) {
            unsafe {
                let ret = dlclose(self.handle);
                if ret != 0 {
                    panic!("Error while closing lib");
                }
            }
        }
    }
}
// pub fn run_update(scripts: &Vec<Script>, world_state: &tar_ecs::World) {
//
// }