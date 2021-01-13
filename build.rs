fn main() {
    cxx_build::bridge("src/lib.rs")
        .file("src/gabbridge.cc")
        .compile("gaboratorrs");

    println!("cargo:rerun-if-changed=src/lib.rs");
    println!("cargo:rerun-if-changed=src/gabbridge.cc");
    println!("cargo:rerun-if-changed=src/gabbridge.h");
}
