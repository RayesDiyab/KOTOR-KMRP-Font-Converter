// KMRP Font Converter: the eleven tiles, one for each distinct font the game draws its text in.

use std::collections::HashMap;
use std::sync::Arc;

use crate::font::{FontFaceChoice, LoadedFace};

pub struct Piece {
    pub name: String,
    pub resref: String,
    pub text: String,
    pub height: f32,
    pub colour: [u8; 3],
}

pub struct TileDef {
    pub key: &'static str,
    pub name: &'static str,
    pub group: &'static str,
    pub use_: &'static str,
    pub resrefs: &'static [&'static str],
    pub centre: bool,
    pub lines: Vec<Vec<Piece>>,
}

#[derive(Default)]
pub struct TileState {
    pub choice: Option<FontFaceChoice>,
    pub face: Option<Arc<LoadedFace>>,
    /// Every face of the picked file, for the picker.
    pub file_faces: Option<Vec<FontFaceChoice>>,
    pub missing: Vec<char>,
    /// Written since it was picked.
    pub converted: bool,
}

pub struct Tile {
    pub def: TileDef,
    pub state: TileState,
}

// The game's 17 text fonts hold 11 distinct letter sets (KMRP's tools/build_hd_original_otf.py
// finds them by comparing every glyph). A tile is one set; its font is written to every name
// that shares it, so text the game draws in either name changes together.
const DEFS: [(&str, &str, &str, &str, bool, &[&str]); 11] = [
    ("menu", "Menu", "Menus", "Screen titles and buttons", false, &["dialogfont16x16", "dialogfont16x16b"]),
    ("menusmall", "Menu Small", "Menus", "Option lists and small labels", false, &["dialogfont10x10", "dialogfont10x10a", "dialogfont16x16a"]),
    ("menusmallb", "Menu Small Bold", "Menus", "Tabs and bold labels", false, &["dialogfont10x10b"]),
    ("menumedium", "Menu Medium", "Menus", "Save and load lists", false, &["dialogfont12x16"]),
    ("menularge", "Menu Large", "Menus", "The biggest headings", true, &["dialogfont32x32"]),
    ("dialogue", "Dialogue", "Reading", "What people say, and subtitles", false, &["fnt_d16x16", "fnt_d10x10b"]),
    ("descriptions", "Replies", "Reading", "Your answers, item and skill text", false, &["fnt_d16x16b"]),
    ("light", "Journal", "Reading", "The journal and long reading", false, &["fnt_d16x16a", "fnt_creditsa", "fnt_dialog16x16"]),
    ("console", "HUD", "HUD", "Credits, level and XP", false, &["fnt_console"]),
    ("credits", "Credits", "Credits", "Job titles", true, &["fnt_credits"]),
    ("creditsb", "Credits Bold", "Credits", "Names", true, &["fnt_creditsb"]),
];

pub fn all(examples: &str) -> Vec<Tile> {
    let mut defs: Vec<TileDef> = DEFS
        .iter()
        .map(|&(key, name, group, use_, centre, resrefs)| TileDef { key, name, group, use_, resrefs, centre, lines: Vec::new() })
        .collect();
    let index: HashMap<&str, usize> = defs.iter().enumerate().map(|(i, d)| (d.key, i)).collect();
    let mut tile: Option<usize> = None;
    for raw in examples.split('\n') {
        let line = raw.trim_end_matches('\r');
        if let Some(key) = line.strip_prefix("tile ") {
            tile = index.get(key).copied();
        } else if line.trim() == "line" {
            if let Some(t) = tile {
                defs[t].lines.push(Vec::new());
            }
        } else if line.trim_start().starts_with("piece ") {
            let p: Vec<&str> = line.trim().splitn(8, ' ').collect();
            if let (Some(t), 8) = (tile, p.len()) {
                let num = |s: &str| s.parse::<u8>().unwrap_or(255);
                let piece = Piece {
                    name: p[1].to_string(),
                    resref: p[2].to_string(),
                    height: p[3].parse().unwrap_or(16.0),
                    colour: [num(p[4]), num(p[5]), num(p[6])],
                    text: p[7].to_string(),
                };
                if let Some(l) = defs[t].lines.last_mut() {
                    l.push(piece);
                }
            }
        }
    }
    defs.into_iter().map(|def| Tile { def, state: TileState::default() }).collect()
}
