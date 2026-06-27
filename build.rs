fn main() {
    if cfg!(target_os = "macos") {
        println!("cargo:rustc-link-arg=-ObjC");
    };
}
