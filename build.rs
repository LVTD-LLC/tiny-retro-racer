use std::env;

fn main() {
    println!("cargo:rerun-if-changed=assets/app-icon.ico");
    println!("cargo:rerun-if-changed=Cargo.toml");

    if env::var("CARGO_CFG_TARGET_OS").as_deref() != Ok("windows") {
        return;
    }

    winresource::WindowsResource::new()
        .set_icon("assets/app-icon.ico")
        .set("ProductName", "Tiny Retro Racer")
        .set("FileDescription", "Tiny Retro Racer")
        .compile()
        .expect("failed to embed Windows application icon");
}
