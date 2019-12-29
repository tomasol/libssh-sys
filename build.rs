fn main() {
    // dynamic linking
    if let Ok(lib) = pkg_config::find_library("libssh") {
        for path in &lib.include_paths {
            println!("cargo:include={}", path.display());
        }
        return;
    }
}
