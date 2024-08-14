use std::env;
use std::fs;
use std::path::PathBuf;

fn main() {
    let out_dir = env::var("OUT_DIR").unwrap();
    let target_dir = PathBuf::from(out_dir)
        .ancestors()
        .nth(4)
        .unwrap()
        .to_path_buf();
    let profile = env::var("PROFILE").expect("Failed to get profile");

    let src_path = PathBuf::from("src");
    let dest_path = target_dir.join(profile);

    fs::copy(src_path.join("hello.html"), dest_path.join("hello.html")).expect("Failed to copy file");
    fs::copy(src_path.join("404.html"), dest_path.join("404.html")).expect("Failed to copy file");
}
