fn main() {
    tauri_build::build();
    if let Ok(out_dir) = std::env::var("OUT_DIR") {
        let resource_lib = format!("{}/libresource.a", out_dir.replace('\\', "/"));
        println!("cargo:rustc-link-arg-tests={}", resource_lib);
    }
}
