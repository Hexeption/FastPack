use std::{env, fs, io::BufReader, path::PathBuf};

fn main() {
    let manifest_dir = PathBuf::from(env::var("CARGO_MANIFEST_DIR").unwrap());
    let ico_path = manifest_dir.join("../../assets/icon.ico");
    let out = PathBuf::from(env::var("OUT_DIR").unwrap());

    println!("cargo:rerun-if-changed=../../assets/icon.ico");

    if ico_path.exists() {
        let file = BufReader::new(fs::File::open(&ico_path).expect("open icon.ico"));
        let dir = ico::IconDir::read(file).expect("parse icon.ico");
        let entry = dir
            .entries()
            .iter()
            .max_by_key(|e| e.width())
            .expect("icon.ico is empty");
        let image = entry.decode().expect("decode icon entry");
        let (w, h) = (image.width(), image.height());
        let rgba = image.rgba_data().to_vec();
        fs::write(out.join("icon.rgba"), &rgba).expect("write icon.rgba");
        fs::write(
            out.join("icon_meta.rs"),
            format!("const ICON_WIDTH: u32 = {w};\nconst ICON_HEIGHT: u32 = {h};\n"),
        )
        .expect("write icon_meta.rs");
    } else {
        // No icon available (e.g. during `cargo publish` verification).
        // Write a 1×1 transparent placeholder so the crate still compiles.
        fs::write(out.join("icon.rgba"), [0u8, 0, 0, 0]).expect("write icon.rgba");
        fs::write(
            out.join("icon_meta.rs"),
            "const ICON_WIDTH: u32 = 1;\nconst ICON_HEIGHT: u32 = 1;\n",
        )
        .expect("write icon_meta.rs");
    }
}
