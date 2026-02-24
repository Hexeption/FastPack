use std::{env, fs, io::BufReader, path::PathBuf};

fn main() {
    println!("cargo:rerun-if-changed=../../assets/icon.ico");

    let out = PathBuf::from(env::var("OUT_DIR").unwrap());
    let ico_path = "../../assets/icon.ico";

    let file = BufReader::new(fs::File::open(ico_path).expect("assets/icon.ico not found"));
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
        format!(
            "const ICON_WIDTH: u32 = {};\nconst ICON_HEIGHT: u32 = {};\n",
            w, h
        ),
    )
    .expect("write icon_meta.rs");
}
