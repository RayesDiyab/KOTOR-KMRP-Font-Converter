// Build script: the tiles' example pictures listed for include_bytes!, and on Windows the
// program's icon and version in the exe.
use std::fmt::Write as _;
use std::path::Path;

fn main() {
    let here = std::env::var("CARGO_MANIFEST_DIR").unwrap();
    let examples = Path::new(&here).join("data").join("examples");
    let mut names: Vec<String> = std::fs::read_dir(&examples)
        .expect("data/examples")
        .flatten()
        .map(|e| e.file_name().to_string_lossy().into_owned())
        .filter(|n| n.ends_with(".png"))
        .collect();
    names.sort();
    let mut code = String::from("pub static EXAMPLE_PNGS: &[(&str, &[u8])] = &[\n");
    for n in &names {
        let path = examples.join(n).to_string_lossy().replace('\\', "/");
        writeln!(code, "    ({:?}, include_bytes!({:?})),", n.trim_end_matches(".png"), path).unwrap();
    }
    code.push_str("];\n");
    let out = Path::new(&std::env::var("OUT_DIR").unwrap()).join("examples.rs");
    std::fs::write(out, code).unwrap();
    println!("cargo:rerun-if-changed=data/examples");
    println!("cargo:rerun-if-changed=favicon.ico");

    if std::env::var("CARGO_CFG_TARGET_OS").as_deref() == Ok("windows") {
        windows_resources();
    }
}

#[cfg(windows)]
fn windows_resources() {
    let mut res = winresource::WindowsResource::new();
    res.set_icon("favicon.ico");
    res.set("ProductName", "KMRP Font Converter");
    res.set("FileDescription", "KMRP Font Converter");
    res.set("CompanyName", "RaymanGT");
    res.set("LegalCopyright", "Copyright (c) 2026 RaymanGT. GPL-3.0-or-later.");
    res.set("OriginalFilename", "KMRP Font Converter.exe");
    if let Err(e) = res.compile() {
        // Without the Windows SDK's resource compiler the exe has no icon; it still runs.
        println!("cargo:warning=no icon in the exe: {e}");
    }
}

#[cfg(not(windows))]
fn windows_resources() {}
