use std::{
    env, fs,
    path::{Path, PathBuf},
};

fn main() {
    generate_embedded_web_assets();
    tauri_build::build()
}

fn generate_embedded_web_assets() {
    let out_dir = PathBuf::from(env::var("OUT_DIR").expect("OUT_DIR 未设置"));
    let output = out_dir.join("web_assets.rs");
    let manifest_dir =
        PathBuf::from(env::var("CARGO_MANIFEST_DIR").expect("CARGO_MANIFEST_DIR 未设置"));
    let dist_dir = manifest_dir.join("..").join("dist");

    println!("cargo:rerun-if-changed={}", dist_dir.display());

    let mut entries = Vec::new();
    if dist_dir.is_dir() {
        collect_files(&dist_dir, &dist_dir, &mut entries);
    }

    let mut generated = String::from("static WEB_ASSETS: &[WebAsset] = &[\n");
    for (relative_path, absolute_path) in entries {
        println!("cargo:rerun-if-changed={}", absolute_path.display());
        let normalized_path = relative_path.replace('\\', "/");
        let content_type = content_type_for(&normalized_path);
        generated.push_str(&format!(
            "    WebAsset {{ path: {:?}, content_type: {:?}, content: include_bytes!({:?}) }},\n",
            normalized_path,
            content_type,
            absolute_path.to_string_lossy().to_string(),
        ));
    }
    generated.push_str("];\n");

    fs::write(output, generated).expect("无法生成 Web 静态资源清单");
}

fn collect_files(root: &Path, current: &Path, entries: &mut Vec<(String, PathBuf)>) {
    let Ok(read_dir) = fs::read_dir(current) else {
        return;
    };

    for entry in read_dir.flatten() {
        let path = entry.path();
        if path.is_dir() {
            collect_files(root, &path, entries);
        } else if path.is_file()
            && let Ok(relative) = path.strip_prefix(root)
        {
            entries.push((relative.to_string_lossy().to_string(), path));
        }
    }

    entries.sort_by(|left, right| left.0.cmp(&right.0));
}

fn content_type_for(path: &str) -> &'static str {
    match Path::new(path)
        .extension()
        .and_then(|value| value.to_str())
        .unwrap_or_default()
    {
        "html" => "text/html; charset=utf-8",
        "js" => "text/javascript; charset=utf-8",
        "css" => "text/css; charset=utf-8",
        "json" => "application/json; charset=utf-8",
        "svg" => "image/svg+xml",
        "png" => "image/png",
        "jpg" | "jpeg" => "image/jpeg",
        "ico" => "image/x-icon",
        "woff" => "font/woff",
        "woff2" => "font/woff2",
        "map" => "application/json; charset=utf-8",
        _ => "application/octet-stream",
    }
}
