#![allow(unused)]

#[cxx::bridge]
mod ffi {
    unsafe extern "C++" {
        include!("gaborator/src/gabbridge.h");

        type GabBridge;

        fn new_gabbridge() -> UniquePtr<GabBridge>;
    }
}
