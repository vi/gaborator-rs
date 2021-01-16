fn main() {
    cxx_build::bridge("src/lib.rs")
        .flag_if_supported("-Wno-type-limits")
        .flag_if_supported("-Wno-deprecated-copy")
        .file("src/gabbridge.cc")
        .compile("gaboratorrs");

    println!("cargo:rerun-if-changed=src/lib.rs");
    println!("cargo:rerun-if-changed=src/gabbridge.cc");
    println!("cargo:rerun-if-changed=src/gabbridge.h");
}
