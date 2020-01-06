extern crate bindgen;
// based on https://rust-lang.github.io/rust-bindgen/tutorial-0.html
use std::env;
use std::fs::OpenOptions;
use std::io::prelude::*;
use std::path::PathBuf;

fn main() {
    // Tell cargo to tell rustc to link the system libssh
    // shared library.
    println!("cargo:rustc-link-lib=ssh");

    // Tell cargo to invalidate the built crate whenever the wrapper changes
    println!("cargo:rerun-if-changed=wrapper.h");

    // The bindgen::Builder is the main entry point
    // to bindgen, and lets you build up options for
    // the resulting bindings.
    let bindings = bindgen::Builder::default()
        // The input header we would like to generate
        // bindings for.
        .header("wrapper.h")
        // Tell cargo to invalidate the built crate whenever any of the
        // included header files changed.
        .parse_callbacks(Box::new(bindgen::CargoCallbacks))
        // Finish the builder and generate the bindings.
        .generate()
        // Unwrap the Result and panic on failure.
        .expect("Unable to generate bindings");

    // Write the bindings to the $OUT_DIR/bindings.rs file.
    let out_path = PathBuf::from(env::var("OUT_DIR").unwrap());
    let mut generated = bindings.to_string();
    // following declaration is generated twice, comment out second occurance
    if let Some(pos) = generated.find("pub const IPPORT_RESERVED: _bindgen_ty_9 = 1024;") {
        generated.replace_range(pos..pos, "// FIXME: duplicate:");
    }
    let out_path = out_path.join("bindings.rs");
    let mut out_file = OpenOptions::new()
        .write(true)
        .truncate(true)
        .create(true)
        .open(out_path)
        .expect("Unable to open file");
    out_file
        .write_all(generated.as_bytes())
        .expect("Unable to write to file");
}
