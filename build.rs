use std::{env, ffi::OsString, path::PathBuf, process::Command};

fn main() {
    println!("cargo:rerun-if-env-changed=FFMPEG_LIB_DIR");
    println!("cargo:rerun-if-env-changed=PKG_CONFIG");
    println!("cargo:rerun-if-env-changed=PKG_CONFIG_PATH");

    if let Some(lib_dir) = ffmpeg_lib_dir() {
        println!("cargo:rustc-link-search=native={}", lib_dir.display());
    } else {
        println!(
            "cargo:warning=FFmpeg library directory was not found; set FFMPEG_LIB_DIR or configure pkg-config"
        );
    }
}

fn ffmpeg_lib_dir() -> Option<PathBuf> {
    if let Some(lib_dir) = env::var_os("FFMPEG_LIB_DIR").filter(|value| !value.is_empty()) {
        return Some(PathBuf::from(lib_dir));
    }

    let pkg_config = env::var_os("PKG_CONFIG").unwrap_or_else(|| OsString::from("pkg-config"));
    let output = Command::new(pkg_config)
        .args(["--variable=libdir", "libavcodec"])
        .output()
        .ok()?;

    if !output.status.success() {
        return None;
    }

    let lib_dir = String::from_utf8(output.stdout).ok()?;
    let lib_dir = lib_dir.trim();

    (!lib_dir.is_empty()).then(|| PathBuf::from(lib_dir))
}
