fn main() {
    println!("cargo:rerun-if-changed=../../assets/icon.ico");

    #[cfg(target_os = "windows")]
    embed_if_present();
}

#[cfg(target_os = "windows")]
fn embed_if_present() {
    use std::{env, path::PathBuf};
    let manifest_dir = PathBuf::from(env::var("CARGO_MANIFEST_DIR").unwrap());
    let ico_path = manifest_dir.join("../../assets/icon.ico");
    if ico_path.exists() {
        winresource::WindowsResource::new()
            .set_icon(ico_path.to_str().expect("icon path is not valid UTF-8"))
            .compile()
            .expect("winresource compile failed");
    }
}
