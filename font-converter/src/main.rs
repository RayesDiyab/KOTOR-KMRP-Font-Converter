// KMRP Font Converter: any font file into KOTOR's game fonts, for KMRP or the original game.
//
// The program's start, its one version number (Cargo.toml's, read here as VERSION) and its
// embedded files. The window is app.rs (its look theme.rs), the tiles tiles.rs, settings and
// the game's folder places.rs, writing the fonts conversion.rs and atlas.rs, reading font
// files font.rs.
//
// Rewritten in Rust on 2026-11-09 from the C# (.NET Framework 4, Windows only) program, so
// one source builds for Windows, macOS and Linux. The fonts it writes for KMRP are made by
// the same arithmetic; see docs/font-converter.md section 9.
#![cfg_attr(all(windows, not(debug_assertions)), windows_subsystem = "windows")]

mod app;
mod atlas;
mod conversion;
mod font;
mod places;
mod theme;
mod tiles;

use std::path::PathBuf;
use std::sync::Arc;

use atlas::Target;

pub const VERSION: &str = env!("CARGO_PKG_VERSION");

pub const REFERENCES: &str = include_str!("../data/references.txt");
pub const EXAMPLES: &str = include_str!("../data/examples.txt");
pub const KMRP_LOGO: &[u8] = include_bytes!("../kmrp-logo.png");
// The same lockup saying KOTOR, for fonts made for the original game (2026-11-08).
pub const KOTOR_LOGO: &[u8] = include_bytes!("../kotor-logo.png");
// The installer's settings gear, line art drawn tinted.
pub const GEAR: &[u8] = include_bytes!("../settings.png");
// The program's icon: the sigil above FONT.
pub const FONT_LOGO: &[u8] = include_bytes!("../font-logo.png");
// KMRP's own fonts for the window's text (SIL OFL 1.1, fonts/OFL.txt).
pub const MENU_FONT: &[u8] = include_bytes!("../fonts/KMRP-OriginalHD-Menu.ttf");
pub const BODY_FONT: &[u8] = include_bytes!("../fonts/KMRP-OriginalHD-Descriptions.ttf");
// The tiles' examples drawn in the HD Original font (make_data.py): (name, PNG).
include!(concat!(env!("OUT_DIR"), "/examples.rs"));

/// A tile's font from the command line: `key=path[#face]`, or `--pick key=path`.
pub struct FontArg {
    pub key: String,
    pub path: PathBuf,
    pub face: Option<String>,
    pub pick: bool,
}

/// For screenshots, checks and tests/Test-FontConverter.py:
///   key=path[#face]       a tile's font (key as in tiles.rs, face by name)
///   --pick key=path       open the picker for a tile (one argument, quoted)
///   --out=folder          write to this folder (not remembered)
///   --version=kmrp|original   make the fonts for this version (not remembered)
///   --textsize=small|medium|large, --theme=dark|light   the look (not remembered)
///   --convert             convert as soon as the window shows
///   --exit                with --convert: no window; convert, then exit 0 or 1
///   --size=WxH            the window's inside at this size, in pixels
///   --still               ignore the mouse (screenshots)
///   --settings            open Settings
#[derive(Default)]
pub struct Args {
    pub out: Option<PathBuf>,
    pub text_size: Option<usize>,
    pub light: Option<bool>,
    pub version: Option<Target>,
    pub size: Option<(u32, u32)>,
    pub convert: bool,
    pub exit: bool,
    pub still: bool,
    pub settings: bool,
    pub fonts: Vec<FontArg>,
}

impl Args {
    fn parse(args: impl Iterator<Item = String>) -> Args {
        let mut a = Args::default();
        for arg in args {
            if let Some(v) = arg.strip_prefix("--out=") {
                a.out = Some(PathBuf::from(v));
            } else if let Some(v) = arg.strip_prefix("--textsize=") {
                a.text_size = places::TEXT_SIZES.iter().position(|s| *s == v).or(a.text_size);
            } else if arg == "--theme=light" || arg == "--theme=dark" {
                a.light = Some(arg == "--theme=light");
            } else if arg == "--version=kmrp" {
                a.version = Some(Target::Kmrp);
            } else if arg == "--version=original" {
                a.version = Some(Target::Original);
            } else if let Some(v) = arg.strip_prefix("--size=") {
                let wh: Vec<&str> = v.split('x').collect();
                if let [w, h] = wh[..] {
                    if let (Ok(w), Ok(h)) = (w.parse::<u32>(), h.parse::<u32>()) {
                        if w > 0 && h > 0 {
                            a.size = Some((w, h));
                        }
                    }
                }
            } else if arg == "--convert" {
                a.convert = true;
            } else if arg == "--exit" {
                a.exit = true;
            } else if arg == "--still" {
                a.still = true;
            } else if arg == "--settings" {
                a.settings = true;
            } else {
                let pick = arg.starts_with("--pick ");
                let spec = if pick { &arg[7..] } else { &arg[..] };
                let Some(eq) = spec.find('=') else { continue };
                let (key, rest) = (&spec[..eq], &spec[eq + 1..]);
                let (path, face) = match rest.find('#') {
                    Some(h) => (&rest[..h], Some(rest[h + 1..].to_string())),
                    None => (rest, None),
                };
                a.fonts.push(FontArg { key: key.to_string(), path: PathBuf::from(path), face, pick });
            }
        }
        a
    }
}

fn main() {
    let args = Args::parse(std::env::args().skip(1));
    if args.convert && args.exit {
        std::process::exit(headless(args));
    }
    std::process::exit(app::run(args));
}

/// `--convert --exit`: the conversion without a window, as the window would make it, for the
/// test and for scripts. Messages go to the standard error; the exit code is 0, or 1 when
/// nothing was converted or it failed. No Replace question, as with the C# program's
/// `--exit`.
fn headless(args: Args) -> i32 {
    let references = Arc::new(atlas::Reference::parse(REFERENCES));
    let mut tiles = tiles::all(EXAMPLES);
    let message = |title: &str, body: &str| eprintln!("{title}: {body}");
    for spec in &args.fonts {
        let Some(tile) = tiles.iter_mut().find(|t| t.def.key == spec.key) else { continue };
        if spec.pick && !font::accepted(&spec.path) {
            message("This font type can't be used", &app::not_a_font_type(&spec.path));
            continue;
        }
        let faces = match font::faces(&spec.path) {
            Ok(f) => f,
            Err(e) => {
                message("This font can't be read", &e.0);
                continue;
            }
        };
        let choice = faces.iter().find(|f| spec.face.as_ref().is_none_or(|n| &f.name == n)).unwrap_or(&faces[0]).clone();
        match app::checked_face(&choice) {
            Ok((face, missing)) => {
                tile.state = tiles::TileState { choice: Some(choice), face: Some(face), file_faces: Some(faces), missing, converted: false };
            }
            Err((title, body)) => message(&title, &body),
        }
    }
    let chosen: Vec<&tiles::Tile> = tiles.iter().filter(|t| t.state.face.is_some()).collect();
    if chosen.is_empty() {
        return 1;
    }
    let output = args.out.clone().unwrap_or_else(|| places::saved_folder().unwrap_or_else(places::default_folder));
    let target = args.version.unwrap_or_else(|| places::load_target(&output));
    let mut job = conversion::Conversion {
        folder: output,
        target,
        references,
        work: chosen
            .iter()
            .flat_map(|t| t.def.resrefs.iter().map(|r| (r.to_string(), t.state.face.clone().unwrap())))
            .collect(),
        progress: Default::default(),
        stop: Default::default(),
        moving: false,
    };
    match job.run(&|| {}) {
        Ok(()) => 0,
        Err(conversion::Failure::Stopped) => 1,
        Err(conversion::Failure::Failed(m)) => {
            message("The fonts could not be written", &app::write_failure(&m, job.moving));
            1
        }
    }
}
