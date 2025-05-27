// build.rs
fn main() {
    // Configura las características de compilación según el objetivo
    if std::env::var("TARGET").unwrap().contains("wasm32-unknown-unknown") {
        println!("cargo:rustc-cfg=wasm32");
        println!("cargo:rustc-cfg=feature=\"wasm\"");
    } else {
        println!("cargo:rustc-cfg=feature=\"native\"");
    }
}
