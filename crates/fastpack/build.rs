fn main() {
    println!("cargo:rerun-if-changed=../../assets/icon.ico");
    #[cfg(target_os = "windows")]
    embed_icon_resource();
}

#[cfg(target_os = "windows")]
fn embed_icon_resource() {
    winresource::WindowsResource::new()
        .set_icon("../../assets/icon.ico")
        .compile()
        .expect("winresource compile failed");
}
