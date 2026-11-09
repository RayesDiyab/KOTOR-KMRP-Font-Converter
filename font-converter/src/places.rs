// KMRP Font Converter: the settings file, and where the game and its Override folder are.
//
// settings.txt: key=value lines. output (the folder), textsize (small, medium, large), theme
// (dark, light) and version (kmrp, original). Kept in the system's settings folder:
// %APPDATA%\KMRP Font Converter on Windows (where the C# program kept it),
// ~/Library/Application Support/KMRP Font Converter on macOS, and $XDG_CONFIG_HOME (or
// ~/.config)/KMRP Font Converter on Linux.

use std::collections::BTreeMap;
use std::path::{Path, PathBuf};

use crate::atlas::Target;

const APP: &str = "KMRP Font Converter";

fn settings_file() -> Option<PathBuf> {
    dirs::config_dir().map(|d| d.join(APP).join("settings.txt"))
}

fn settings() -> BTreeMap<String, String> {
    let mut values = BTreeMap::new();
    if let Some(text) = settings_file().and_then(|f| std::fs::read_to_string(f).ok()) {
        for line in text.lines() {
            if let Some(eq) = line.find('=') {
                if eq > 0 {
                    values.insert(line[..eq].to_string(), line[eq + 1..].to_string());
                }
            }
        }
    }
    values
}

pub fn setting(key: &str) -> Option<String> {
    settings().remove(key)
}

pub fn set(key: &str, value: &str) {
    let Some(file) = settings_file() else { return };
    let mut values = settings();
    values.insert(key.to_string(), value.to_string());
    if let Some(dir) = file.parent() {
        let _ = std::fs::create_dir_all(dir);
    }
    let text: String = values.iter().map(|(k, v)| format!("{k}={v}\n")).collect();
    let _ = std::fs::write(file, text);
}

pub fn saved_folder() -> Option<PathBuf> {
    setting("output").map(PathBuf::from).filter(|p| p.is_dir())
}

pub fn save_folder(folder: &Path) {
    set("output", &folder.to_string_lossy());
}

pub const TEXT_SIZES: [&str; 3] = ["small", "medium", "large"];
pub const TEXT_SCALES: [f32; 3] = [1.0, 1.2, 1.32];

/// The text size (an index into TEXT_SCALES) and whether the theme is light, from settings.txt.
pub fn load_look() -> (usize, bool) {
    let size = setting("textsize").and_then(|s| TEXT_SIZES.iter().position(|t| *t == s)).unwrap_or(1);
    (size, setting("theme").as_deref() == Some("light"))
}

/// The version the fonts are made for: as last chosen, else KMRP when KMRP is installed in
/// the game the output folder belongs to (or, with no game found there, in any game found),
/// else the original game.
pub fn load_target(output: &Path) -> Target {
    match setting("version").as_deref() {
        Some("kmrp") => return Target::Kmrp,
        Some("original") => return Target::Original,
        _ => {}
    }
    if let Some(game) = output.parent() {
        if game.join("swkotor.exe").is_file() {
            return if has_kmrp(game) { Target::Kmrp } else { Target::Original };
        }
    }
    if games().iter().any(|g| g.join("swkotor.exe").is_file() && has_kmrp(g)) {
        Target::Kmrp
    } else {
        Target::Original
    }
}

pub fn save_target(target: Target) {
    set("version", if target == Target::Kmrp { "kmrp" } else { "original" });
}

// KMRP's installer leaves its folder and its module in the game's folder.
fn has_kmrp(game: &Path) -> bool {
    game.join("patches").join("kmrp.dll").is_file() || game.join("KMRP").is_dir()
}

/// The game's Override folder when the game can be found (Steam or GOG), else a folder in
/// Documents.
pub fn default_folder() -> PathBuf {
    for game in games() {
        if game.join("swkotor.exe").is_file() {
            return cased(&game).join("Override");
        }
    }
    dirs::document_dir().or_else(dirs::home_dir).unwrap_or_else(|| PathBuf::from(".")).join(APP)
}

// A folder's name as the disk spells it (Steam keeps its own path in lower case on Windows).
fn cased(path: &Path) -> PathBuf {
    if !cfg!(windows) {
        return path.to_path_buf();
    }
    let mut out = PathBuf::new();
    for part in path.components() {
        let name = part.as_os_str();
        if !matches!(part, std::path::Component::Normal(_)) {
            out.push(name.to_string_lossy().to_uppercase());
            continue;
        }
        let found = std::fs::read_dir(if out.as_os_str().is_empty() { Path::new(".") } else { &out })
            .ok()
            .and_then(|entries| {
                entries
                    .flatten()
                    .map(|e| e.file_name())
                    .find(|f| f.to_string_lossy().eq_ignore_ascii_case(&name.to_string_lossy()))
            });
        match (part, found) {
            (std::path::Component::Normal(_), Some(f)) => out.push(f),
            _ => out.push(part.as_os_str()),
        }
    }
    out
}

/// The folder the program is in, as the player sees it: beside the AppImage, beside the
/// macOS app, or the exe's own.
fn program_folder() -> Option<PathBuf> {
    if let Some(appimage) = std::env::var_os("APPIMAGE") {
        return PathBuf::from(appimage).parent().map(Path::to_path_buf);
    }
    let exe = std::env::current_exe().ok()?;
    let mut dir = exe.parent()?.to_path_buf();
    if let Some(app) = dir.ancestors().find(|a| a.extension().is_some_and(|e| e == "app")) {
        dir = app.parent()?.to_path_buf();
    }
    Some(dir)
}

/// Steam's own folders, where the platform puts them.
fn steam_roots() -> Vec<PathBuf> {
    let mut roots = Vec::new();
    #[cfg(windows)]
    {
        use winreg::enums::HKEY_CURRENT_USER;
        if let Ok(key) = winreg::RegKey::predef(HKEY_CURRENT_USER).open_subkey("Software\\Valve\\Steam") {
            if let Ok(path) = key.get_value::<String, _>("SteamPath") {
                roots.push(PathBuf::from(path.replace('/', "\\")));
            }
        }
    }
    if let Some(home) = dirs::home_dir() {
        if cfg!(target_os = "macos") {
            roots.push(home.join("Library/Application Support/Steam"));
        } else if cfg!(target_os = "linux") {
            roots.push(home.join(".steam/steam"));
            roots.push(home.join(".local/share/Steam"));
            roots.push(home.join(".var/app/com.valvesoftware.Steam/.local/share/Steam"));
        }
    }
    roots
}

fn games() -> Vec<PathBuf> {
    let mut found = Vec::new();
    // The folder the converter itself is in, and the one above it: a player who puts it in
    // the game's folder (a disc or other install no registry key names) has the game found
    // that way.
    if let Some(here) = program_folder() {
        if let Some(up) = here.parent() {
            found.push(here.clone());
            found.push(up.to_path_buf());
        } else {
            found.push(here);
        }
    }
    for steam in steam_roots() {
        let mut libraries = vec![steam.clone()];
        if let Ok(vdf) = std::fs::read_to_string(steam.join("steamapps").join("libraryfolders.vdf")) {
            for line in vdf.lines() {
                let t = line.trim();
                if !t.starts_with("\"path\"") {
                    continue;
                }
                let parts: Vec<&str> = t.split('"').collect();
                if parts.len() >= 4 {
                    libraries.push(PathBuf::from(parts[3].replace("\\\\", "\\")));
                }
            }
        }
        for library in libraries {
            found.push(library.join("steamapps").join("common").join("swkotor"));
        }
    }
    #[cfg(windows)]
    {
        use winreg::enums::HKEY_LOCAL_MACHINE;
        for key in ["SOFTWARE\\WOW6432Node\\GOG.com\\Games\\1207666283", "SOFTWARE\\GOG.com\\Games\\1207666283"] {
            if let Ok(k) = winreg::RegKey::predef(HKEY_LOCAL_MACHINE).open_subkey(key) {
                if let Ok(path) = k.get_value::<String, _>("path") {
                    if !path.is_empty() {
                        found.push(PathBuf::from(path));
                    }
                }
            }
        }
        found.push(PathBuf::from("C:\\Program Files (x86)\\Steam\\steamapps\\common\\swkotor"));
        found.push(PathBuf::from("C:\\GOG Games\\Star Wars - KotOR"));
    }
    found
}
