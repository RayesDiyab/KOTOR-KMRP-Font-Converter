// KMRP Font Converter: a font face to the game's font atlases.
//
// The rules are KMRP's tools/build_font_from_ttf.py's, measured from the game's own atlases;
// read that file's notes before changing any of them:
//
//   * A slot is a byte of the game's text in Windows-1252. Control codes and the five bytes
//     the code page leaves undefined are blank.
//   * The engine draws one texel per pixel; `texturewidth * 100` is the atlas's width.
//   * A glyph's cell is its advance and nothing outside it is drawn, so a glyph whose ink
//     reaches past its advance gets a wider cell and is moved right inside it.
//   * No slot is zero wide: the game's word wrap loops until it runs out of memory on a
//     zero-wide glyph. A slot the face has no letter for keeps the width of KMRP's HD
//     Original font's glyph there.
//   * 4 pixels between glyphs, so filtering never reaches the next row's ink.
//
// For KMRP the atlases are made at the HD Original font's size, six times the original
// fonts', from its TXI headers: the same heights, baselines and spacing, so text keeps its
// place and size in every interface, and KMRP fits them to the size it draws them at. For
// the original game they are made at the original fonts' own size, a sixth of that: the
// TXI's heights and spacing divided by six (every one divides exactly: 10, 12, 14, 16, 19
// and 32 pixels), and each letter hinted by the font's own instructions, so the small
// letters are crisp. The game draws them one texel to one pixel as it draws its own.
//
// Ported from Atlas.cs on 2026-11-09; the KMRP version's arithmetic is unchanged. For the
// original game the hinted letters were drawn by Windows (DirectWrite, GDI classic); they
// are now hinted by skrifa and filled by the rasteriser below (font.rs).

use std::collections::HashMap;
use std::fs::File;
use std::io::{BufWriter, Write};
use std::path::Path;

use crate::font::{Contours, FontError, LoadedFace, P};

/// One of the game's fonts as KMRP's HD Original draws it.
#[derive(Clone, Debug, Default)]
pub struct Reference {
    pub resref: String,
    pub atlas_width: i32,
    pub header: Vec<String>,
    pub widths: Vec<i32>,
}

impl Reference {
    pub fn field(&self, name: &str) -> f64 {
        for line in &self.header {
            let mut parts = line.split(' ');
            if parts.next() == Some(name) {
                if let Some(v) = parts.next() {
                    return v.parse().unwrap_or(0.0);
                }
            }
        }
        0.0
    }

    pub fn num_chars(&self) -> usize {
        self.field("numchars") as usize
    }

    pub fn glyph_height(&self) -> i32 {
        round(self.field("fontheight") * 100.0) as i32
    }

    /// references.txt, made by make_data.py.
    pub fn parse(text: &str) -> HashMap<String, Reference> {
        let mut all = HashMap::new();
        let mut current: Option<Reference> = None;
        for raw in text.split('\n') {
            let line = raw.trim_end_matches('\r');
            if let Some(rest) = line.strip_prefix("font ") {
                if let Some(done) = current.take() {
                    all.insert(done.resref.clone(), done);
                }
                let mut parts = rest.split(' ');
                let resref = parts.next().unwrap_or("").to_string();
                let atlas_width = parts.next().and_then(|s| s.parse().ok()).unwrap_or(0);
                current = Some(Reference { resref, atlas_width, ..Default::default() });
            } else if let Some(rest) = line.strip_prefix("  widths ") {
                if let Some(c) = current.as_mut() {
                    c.widths = rest.split(' ').filter_map(|s| s.parse().ok()).collect();
                }
            } else if line.starts_with("  ") {
                if let Some(c) = current.as_mut() {
                    c.header.push(line.trim().to_string());
                }
            }
        }
        if let Some(done) = current.take() {
            all.insert(done.resref.clone(), done);
        }
        all
    }
}

/// Who the fonts are made for: KMRP, which sizes them, or the original game.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Target {
    Kmrp,
    Original,
}

pub const PADDING: i32 = 4;
/// The original fonts are a sixth of KMRP's HD size.
pub const ORIGINAL_DIVISOR: i32 = 6;
/// The largest atlas written: KMRP's own largest (dialogfont32x32 at six times). A font that
/// needs more is refused rather than written as a picture of a gigabyte or more (audit F4).
pub const MAX_SIZE: i32 = 4096;
/// How much wider, in the original game's pixels, each hinted letter is made: its strokes
/// thicker sideways and its advance as much longer (2026-11-09, the maintainer: "I would
/// prefer if it was like slightly wider instead of slightly narrower" than Windows' drawing
/// of the C# program). Chosen from 0.25, 0.3, 0.35, 0.4 and 0.6 against that drawing.
pub const ORIGINAL_WIDEN: f32 = 0.3;

/// Rounds half to even, as .NET's Math.Round, so the arithmetic is the C# program's.
pub fn round(v: f64) -> f64 {
    let r = v.round();
    if (v - v.trunc()).abs() == 0.5 {
        2.0 * (v / 2.0).round()
    } else {
        r
    }
}

fn roundf(v: f32) -> f32 {
    round(v as f64) as f32
}

// Windows-1252's 0x80 to 0x9F; the rest of its bytes are their own code points.
const CP1252_HIGH: [u32; 32] = [
    0x20AC, 0, 0x201A, 0x0192, 0x201E, 0x2026, 0x2020, 0x2021, 0x02C6, 0x2030, 0x0160, 0x2039, 0x0152, 0, 0x017D, 0,
    0, 0x2018, 0x2019, 0x201C, 0x201D, 0x2022, 0x2013, 0x2014, 0x02DC, 0x2122, 0x0161, 0x203A, 0x0153, 0, 0x017E, 0x0178,
];

/// The character slot `index` holds, or None for a blank one.
pub fn slot_character(index: usize) -> Option<u32> {
    if index < 0x20 || index == 0x7F || index > 0xFF {
        return None;
    }
    if (0x80..0xA0).contains(&index) {
        let c = CP1252_HIGH[index - 0x80];
        return if c == 0 { None } else { Some(c) };
    }
    Some(index as u32)
}

/// The glyph a face draws in a slot: its own, a space for a missing no-break space, a hyphen
/// for a missing soft hyphen; 0 when it has none.
pub fn slot_glyph(face: &LoadedFace, character: u32) -> u16 {
    let mut glyph = face.glyph(character);
    if glyph == 0 && character == 0xA0 {
        glyph = face.glyph(' ' as u32);
    }
    if glyph == 0 && character == 0xAD {
        glyph = face.glyph('-' as u32);
    }
    glyph
}

/// The letters A to Z and a to z the face has no glyph for. A font missing them would turn
/// the game's text invisible (audit F5), so it is not taken.
pub fn missing_basic_latin(face: &LoadedFace) -> String {
    ('A'..='Z').chain('a'..='z').filter(|&c| face.glyph(c as u32) == 0).collect()
}

/// The letters of the game's text the face has no glyph for.
pub fn missing(face: &LoadedFace) -> Vec<char> {
    (0x20..0x100)
        .filter_map(slot_character)
        .filter(|&c| slot_glyph(face, c) == 0)
        .filter_map(char::from_u32)
        .collect()
}

/// What the glyph box holds, in design units above and below the baseline: the face's
/// ascent and descent, or more where a letter of the game's text reaches further (a caron
/// over a capital often does), so no letter is cut off.
pub fn glyph_box(face: &LoadedFace) -> (f32, f32) {
    *face.box_cache.get_or_init(|| {
        let (mut up, mut down) = (face.ascent as f32, face.descent as f32);
        for i in 0x20..0x100 {
            let Some(c) = slot_character(i) else { continue };
            let glyph = slot_glyph(face, c);
            if glyph == 0 {
                continue;
            }
            for contour in face.outline(glyph, face.units_per_em as f32) {
                for p in contour {
                    up = up.max(-p.y);
                    down = down.max(p.y);
                }
            }
        }
        (up, down)
    })
}

/// Fonts the game draws with a black ring round every letter, and its width at the atlas's
/// size: fnt_console, the HUD's numbers, one original pixel (six at the HD size), as KMRP's
/// tools/build_hd_original_fonts.py (OUTLINED) gives KMRP's own.
pub fn outlined(resref: &str) -> Option<f32> {
    if resref == "fnt_console" {
        Some(6.0)
    } else {
        None
    }
}

/// A letter drawn hinted, in a box relative to its pen on the baseline (y down).
struct Ink {
    coverage: Vec<u8>,
    left: i32,
    top: i32,
    right: i32,
    bottom: i32,
}

/// The outline made `strength` pixels wider, its strokes thicker sideways only, as FreeType's
/// FT_Outline_EmboldenXY does with no vertical strength: every edge moves out by half the
/// strength along its normal's horizontal part, each point by the miter of its two edges
/// (capped, so a sharp corner does not spike); then the whole is moved right by half, so the
/// left side bearing stays and the right grows by the strength.
pub fn embolden_x(contours: &Contours, strength: f32) -> Contours {
    let area: f32 = contours
        .iter()
        .map(|c| (0..c.len()).map(|i| { let (a, b) = (c[i], c[(i + 1) % c.len()]); a.x * b.y - b.x * a.y }).sum::<f32>())
        .sum();
    let outward = if area >= 0.0 { 1.0 } else { -1.0 };
    let half = strength / 2.0;
    let normal = |a: P, b: P| -> Option<(f32, f32)> {
        let (dx, dy) = (b.x - a.x, b.y - a.y);
        let len = (dx * dx + dy * dy).sqrt();
        if len < 1e-6 { None } else { Some((outward * dy / len, -outward * dx / len)) }
    };
    contours
        .iter()
        .map(|c| {
            let n = c.len();
            (0..n)
                .map(|i| {
                    let p = c[i];
                    // The nearest distinct neighbours, for points the flattening repeated.
                    let prev = (1..n).map(|k| c[(i + n - k) % n]).find(|q| (q.x - p.x).abs() + (q.y - p.y).abs() > 1e-6);
                    let next = (1..n).map(|k| c[(i + k) % n]).find(|q| (q.x - p.x).abs() + (q.y - p.y).abs() > 1e-6);
                    let (Some(prev), Some(next)) = (prev, next) else { return P { x: p.x + half, y: p.y } };
                    let (Some(n1), Some(n2)) = (normal(prev, p), normal(p, next)) else { return P { x: p.x + half, y: p.y } };
                    let d = 1.0 + n1.0 * n2.0 + n1.1 * n2.1;
                    let mx = if d > 0.25 { (n1.0 + n2.0) / d } else { (n1.0 + n2.0) / 0.25 };
                    P { x: p.x + half + half * mx.clamp(-2.0, 2.0), y: p.y }
                })
                .collect()
        })
        .collect()
}

/// The hinted outline filled in the whole pixels round its ink; None when it has no ink.
fn hinted_ink(contours: &Contours) -> Option<Ink> {
    let (mut min_x, mut min_y, mut max_x, mut max_y) = (f32::MAX, f32::MAX, f32::MIN, f32::MIN);
    for p in contours.iter().flatten() {
        min_x = min_x.min(p.x);
        max_x = max_x.max(p.x);
        min_y = min_y.min(p.y);
        max_y = max_y.max(p.y);
    }
    if min_x > max_x || min_y > max_y {
        return None;
    }
    let (left, top) = (min_x.floor() as i32, min_y.floor() as i32);
    let (right, bottom) = (max_x.ceil() as i32, max_y.ceil() as i32);
    let (w, h) = (right - left, bottom - top);
    if w <= 0 || h <= 0 {
        return None;
    }
    if w as i64 * h as i64 > 1 << 22 {
        return None;
    }
    let coverage = fill(contours, w as usize, h as usize, -left as f32, -top as f32);
    if coverage.iter().all(|&v| v == 0) {
        return None;
    }
    Some(Ink { coverage, left, top, right, bottom })
}

/// The atlas and TXI for one font, written into `folder`.
pub fn write(face: &LoadedFace, reference: &Reference, folder: &Path, target: Target) -> Result<(), FontError> {
    let original = target == Target::Original;
    let divisor = if original { ORIGINAL_DIVISOR } else { 1 };
    let height = round(reference.glyph_height() as f64 / divisor as f64) as i32;
    let (top, bottom) = glyph_box(face);
    let scale = height as f32 / (top + bottom).max(1.0);
    let em = face.units_per_em as f32 * scale;
    let ascent = top * scale;
    // The original game's letters stand on a whole pixel, where hinting puts them.
    let baseline = roundf(ascent) as i32;
    let count = reference.num_chars();
    let ring = outlined(&reference.resref);
    let ring_width = ring.unwrap_or(0.0) / divisor as f32;

    let mut widths = vec![0i32; count];
    let mut offsets = vec![0f32; count];
    let mut outlines: Vec<Option<Contours>> = vec![None; count];
    let mut hinted: Vec<Option<Ink>> = (0..count).map(|_| None).collect();
    let mut hinter = if original { face.hinter(em) } else { None };
    let mut i = 0;
    while i < count {
        let glyph = slot_character(i).map(|c| slot_glyph(face, c)).unwrap_or(0);
        let mut advance = 0;
        if glyph != 0 {
            // The original game's letters a little wider and fuller than the outline (section
            // 9 of docs/font-converter.md), their advance with them.
            let widen = if original { ORIGINAL_WIDEN } else { 0.0 };
            advance = roundf(face.advance(glyph) as f32 * scale + widen) as i32;
            if let Some(h) = hinter.as_ref() {
                match face.hinted_outline(glyph, h) {
                    Ok(contours) => {
                        let contours = embolden_x(&contours, widen);
                        if let Some(ink) = hinted_ink(&contours) {
                            let left = ink.left.min(0);
                            advance = advance.max(ink.right - left);
                            offsets[i] = -left as f32;
                            hinted[i] = Some(ink);
                        }
                    }
                    // Where the font's hints cannot be run, every letter is drawn unhinted
                    // instead, so the set is one style.
                    Err(_) => {
                        hinter = None;
                        hinted.iter_mut().for_each(|h| *h = None);
                        offsets.iter_mut().for_each(|o| *o = 0.0);
                        widths.iter_mut().for_each(|w| *w = 0);
                        i = 0;
                        continue;
                    }
                }
            } else {
                let outline = face.outline(glyph, em);
                // (for KMRP, widen is 0 and the outline is the font's, as before)
                let outline = if widen > 0.0 { embolden_x(&outline, widen) } else { outline };
                let (mut min_x, mut max_x) = (f32::MAX, f32::MIN);
                for p in outline.iter().flatten() {
                    min_x = min_x.min(p.x);
                    max_x = max_x.max(p.x);
                }
                if min_x <= max_x {
                    let left = min_x.min(0.0);
                    let right = max_x - left;
                    if right > advance as f32 {
                        advance = right.ceil() as i32;
                    }
                    offsets[i] = -left;
                    outlines[i] = Some(outline);
                }
            }
        }
        if advance <= 0 {
            advance = 1.max(match reference.widths.get(i) {
                Some(&w) => round(w as f64 / divisor as f64) as i32,
                None => height / 3,
            });
        }
        // An outlined font's letter gets its ring's width on both sides, so the ring is never
        // cut by the cell.
        if ring.is_some() {
            offsets[i] += ring_width.ceil();
            advance += 2 * ring_width.ceil() as i32;
        }
        if advance > MAX_SIZE {
            return Err(FontError("A letter of this font is too wide for the game's fonts.".into()));
        }
        widths[i] = advance;
        i += 1;
    }

    // Powers of two: the original game's own atlases are, and older drivers want them.
    let mut size = if original { 64 } else { 64.max(reference.atlas_width) };
    let positions = loop {
        if let Some(p) = pack(&widths, height, size) {
            break p;
        }
        size *= 2;
        if size > MAX_SIZE {
            return Err(FontError("This font's letters are too wide to fit the game's fonts.".into()));
        }
    };

    let s = size as usize;
    let mut alpha = vec![0u8; s * s];
    let mut rings = ring.map(|_| vec![0u8; s * s]);
    for i in 0..count {
        let w = widths[i] as usize;
        let h = height as usize;
        let cell = if let Some(ink) = &hinted[i] {
            place(ink, w, h, offsets[i] as i32, baseline)
        } else if let Some(outline) = &outlines[i] {
            fill(outline, w, h, offsets[i], if original { baseline as f32 } else { ascent })
        } else {
            continue;
        };
        let (ax, ay) = (positions[i].0 as usize, positions[i].1 as usize);
        for y in 0..h {
            alpha[(ay + y) * s + ax..(ay + y) * s + ax + w].copy_from_slice(&cell[y * w..y * w + w]);
        }
        if let Some(r) = rings.as_mut() {
            let spread = spread(&cell, w, h, ring_width);
            for y in 0..h {
                r[(ay + y) * s + ax..(ay + y) * s + ax + w].copy_from_slice(&spread[y * w..y * w + w]);
            }
        }
    }

    std::fs::create_dir_all(folder).map_err(|e| FontError(e.to_string()))?;
    write_tga(&folder.join(format!("{}.tga", reference.resref)), size, &alpha, rings.as_deref())
        .map_err(|e| FontError(e.to_string()))?;
    std::fs::write(folder.join(format!("{}.txi", reference.resref)), txi(reference, &positions, &widths, height, size, divisor))
        .map_err(|e| FontError(e.to_string()))?;
    Ok(())
}

/// A hinted letter (box relative to its pen on the baseline) put into its cell: the pen `pen`
/// pixels in, the baseline `baseline` pixels down; any ink past the cell's edge is cut.
fn place(ink: &Ink, width: usize, height: usize, pen: i32, baseline: i32) -> Vec<u8> {
    let mut cell = vec![0u8; width * height];
    let w = (ink.right - ink.left) as usize;
    for y in ink.top..ink.bottom {
        let row = baseline + y;
        if row < 0 || row >= height as i32 {
            continue;
        }
        for x in ink.left..ink.right {
            let col = pen + x;
            if col < 0 || col >= width as i32 {
                continue;
            }
            cell[row as usize * width + col as usize] = ink.coverage[(y - ink.top) as usize * w + (x - ink.left) as usize];
        }
    }
    cell
}

/// A cell's coverage spread by a disc `radius` pixels wide with a soft edge, inside the cell:
/// the letter's black ring.
pub fn spread(cell: &[u8], w: usize, h: usize, radius: f32) -> Vec<u8> {
    let mut out = vec![0u8; w * h];
    let reach = (radius + 1.0).ceil() as i32;
    let (wi, hi) = (w as i32, h as i32);
    for dy in -reach..=reach {
        for dx in -reach..=reach {
            let weight = (radius + 0.5 - ((dx * dx + dy * dy) as f32).sqrt()).clamp(0.0, 1.0);
            if weight <= 0.0 {
                continue;
            }
            for y in dy.max(0)..hi.min(hi + dy) {
                for x in dx.max(0)..wi.min(wi + dx) {
                    let v = (cell[((y - dy) * wi + x - dx) as usize] as f32 * weight) as u8;
                    let o = &mut out[(y * wi + x) as usize];
                    if v > *o {
                        *o = v;
                    }
                }
            }
        }
    }
    out
}

fn pack(widths: &[i32], height: i32, size: i32) -> Option<Vec<(i32, i32)>> {
    let mut positions = Vec::with_capacity(widths.len());
    let (mut x, mut y) = (0, 0);
    for &width in widths {
        if x + width + PADDING > size {
            x = 0;
            y += height + PADDING;
        }
        if y + height > size || width > size {
            return None;
        }
        positions.push((x, y));
        x += width + PADDING;
    }
    Some(positions)
}

// The TXI fields that are lengths in pixels (times 100), divided for the original game.
const LENGTHS: [&str; 4] = ["fontheight", "baselineheight", "spacingR", "spacingB"];

/// A number as the C# program wrote it (.NET Framework's "0.000000"): the value to 15
/// significant digits, then rounded half away from zero to six decimals. Ties are common
/// (2 / 256 is 0.0078125), and Rust's own formatting rounds them to even.
pub fn f(v: f64) -> String {
    if v == 0.0 || !v.is_finite() {
        return "0.000000".into();
    }
    let s = format!("{:.14e}", v.abs());
    let (mantissa, exp) = s.split_once('e').unwrap_or((&s, "0"));
    let exp: i32 = exp.parse().unwrap_or(0);
    let m: i128 = mantissa.bytes().filter(|b| b.is_ascii_digit()).fold(0, |n, b| n * 10 + (b - b'0') as i128);
    // v = m * 10^(exp - 14); in millionths, m * 10^(exp - 8).
    let shift = exp - 8;
    let n: i128 = if shift >= 0 {
        m * 10i128.pow(shift as u32)
    } else if -shift > 30 {
        0
    } else {
        let d = 10i128.pow((-shift) as u32);
        let (q, r) = (m / d, m % d);
        if 2 * r >= d { q + 1 } else { q }
    };
    let sign = if v < 0.0 && n != 0 { "-" } else { "" };
    format!("{}{}.{:06}", sign, n / 1_000_000, n % 1_000_000)
}

fn txi(reference: &Reference, positions: &[(i32, i32)], widths: &[i32], height: i32, size: i32, divisor: i32) -> String {
    let mut b = String::new();
    for line in &reference.header {
        let key = line.split(' ').next().unwrap_or("");
        if key == "texturewidth" {
            b += &format!("texturewidth {}\n", f(size as f64 / 100.0));
        } else if divisor != 1 && LENGTHS.contains(&key) {
            b += &format!("{} {}\n", key, f(reference.field(key) / divisor as f64));
        } else {
            b += line;
            b.push('\n');
        }
    }
    let s = size as f64;
    b += &format!("upperleftcoords {}\n", positions.len());
    for p in positions {
        b += &format!("{} {} 0\n", f(p.0 as f64 / s), f(1.0 - p.1 as f64 / s));
    }
    b += &format!("lowerrightcoords {}\n", positions.len());
    for (i, p) in positions.iter().enumerate() {
        b += &format!("{} {} 0\n", f((p.0 + widths[i]) as f64 / s), f(1.0 - (p.1 + height) as f64 / s));
    }
    b
}

/// Uncompressed 32-bit TGA, bottom-up like the game's own: white ink, the glyphs in alpha.
fn write_tga(path: &Path, size: i32, alpha: &[u8], ring: Option<&[u8]>) -> std::io::Result<()> {
    let mut file = BufWriter::with_capacity(1 << 20, File::create(path)?);
    let mut header = [0u8; 18];
    header[2] = 2;
    header[12] = size as u8;
    header[13] = (size >> 8) as u8;
    header[14] = size as u8;
    header[15] = (size >> 8) as u8;
    header[16] = 32;
    header[17] = 8;
    file.write_all(&header)?;
    let s = size as usize;
    let mut row = vec![0u8; s * 4];
    for y in (0..s).rev() {
        for x in 0..s {
            let (o, i) = (x * 4, y * s + x);
            match ring {
                None => {
                    row[o] = 255;
                    row[o + 1] = 255;
                    row[o + 2] = 255;
                    row[o + 3] = alpha[i];
                }
                Some(ring) => {
                    // White letter over its black ring.
                    let a = alpha[i] as f32 / 255.0;
                    let r = ring[i] as f32 / 255.0;
                    let total = a + r * (1.0 - a);
                    let white = if total > 0.0 { round((255.0 * a / total) as f64) as u8 } else { 0 };
                    row[o] = white;
                    row[o + 1] = white;
                    row[o + 2] = white;
                    row[o + 3] = round((total * 255.0) as f64) as u8;
                }
            }
        }
        file.write_all(&row)?;
    }
    file.flush()
}

/// A line of text in the face, as the tiles show it.
pub struct LineImage {
    pub width: usize,
    pub height: usize,
    /// Premultiplied RGBA.
    pub rgba: Vec<u8>,
    /// The box asked for, and the margin round it.
    pub box_height: i32,
    pub pad: i32,
}

/// A line of text in the face: the glyph box `box_height` pixels high, the face's ascent and
/// descent fitted into it as the atlases are, in `colour` over its black ring, if any.
pub fn line(face: &LoadedFace, text: &str, box_height: f32, colour: [u8; 3], ring_radius: f32) -> LineImage {
    let (top, bottom) = glyph_box(face);
    let scale = box_height / (top + bottom).max(1.0);
    let em = face.units_per_em as f32 * scale;
    let ascent = top * scale;
    let mut glyphs: Vec<(f32, Contours)> = Vec::new();
    let (mut pen, mut min_x, mut max_x) = (0f32, 0f32, 1f32);
    for c in text.chars() {
        let glyph = slot_glyph(face, c as u32);
        if glyph == 0 {
            pen += box_height / 3.0;
            continue;
        }
        let outline = face.outline(glyph, em);
        for p in outline.iter().flatten() {
            min_x = min_x.min(pen + p.x);
            max_x = max_x.max(pen + p.x);
        }
        glyphs.push((pen, outline));
        pen += face.advance(glyph) as f32 * scale;
    }
    max_x = max_x.max(pen);
    let pad = ring_radius.ceil() as i32 + 1;
    let width = (max_x - min_x).ceil() as usize + 2 + 2 * pad as usize;
    let height = box_height.ceil() as usize + 2 + 2 * pad as usize;
    let all: Contours = glyphs
        .iter()
        .flat_map(|(at, outline)| outline.iter().map(move |c| c.iter().map(|p| P { x: p.x + at, y: p.y }).collect()))
        .collect();
    let cover = fill(&all, width, height, 1.0 - min_x + pad as f32, ascent + 1.0 + pad as f32);
    let ring = if ring_radius > 0.0 { Some(spread(&cover, width, height, ring_radius)) } else { None };
    let mut rgba = vec![0u8; width * height * 4];
    for i in 0..width * height {
        // Premultiplied: the coloured letter over its black ring, if any.
        let a = cover[i] as u32;
        let r = ring.as_ref().map(|r| r[i] as u32).unwrap_or(0);
        let total = a + r * (255 - a) / 255;
        rgba[i * 4] = (colour[0] as u32 * a / 255) as u8;
        rgba[i * 4 + 1] = (colour[1] as u32 * a / 255) as u8;
        rgba[i * 4 + 2] = (colour[2] as u32 * a / 255) as u8;
        rgba[i * 4 + 3] = total as u8;
    }
    LineImage { width, height, rgba, box_height: box_height.ceil() as i32, pad }
}

// ---- the rasteriser ----

/// Exact-area coverage of filled outlines: every pixel the share of it the shape covers, so
/// edges are smooth at any size without supersampling.
pub fn fill(contours: &Contours, width: usize, height: usize, dx: f32, dy: f32) -> Vec<u8> {
    let stride = width + 2;
    let mut acc = vec![0f32; stride * height];
    for contour in contours {
        let n = contour.len();
        for i in 0..n {
            let (a, b) = (contour[i], contour[(i + 1) % n]);
            edge(&mut acc, stride, width, height, a.x + dx, a.y + dy, b.x + dx, b.y + dy);
        }
    }
    let mut cover = vec![0u8; width * height];
    for y in 0..height {
        let mut sum = 0f32;
        for x in 0..width {
            sum += acc[y * stride + x];
            let v = sum.abs().min(1.0);
            cover[y * width + x] = (v * 255.0 + 0.5) as u8;
        }
    }
    cover
}

// One edge's signed area into the accumulation buffer (as in font-rs): a cell gets the area
// left of the edge inside it, the running sum along a row then fills.
#[allow(clippy::too_many_arguments)]
fn edge(acc: &mut [f32], stride: usize, width: usize, height: usize, x0: f32, y0: f32, x1: f32, y1: f32) {
    if y0 == y1 {
        return;
    }
    let (mut x0, mut y0, mut x1, mut y1) = (x0, y0, x1, y1);
    let mut dir = 1f32;
    if y0 > y1 {
        dir = -1.0;
        std::mem::swap(&mut x0, &mut x1);
        std::mem::swap(&mut y0, &mut y1);
    }
    let w = width as f32;
    x0 = x0.clamp(0.0, w);
    x1 = x1.clamp(0.0, w);
    let dxdy = (x1 - x0) / (y1 - y0);
    let mut x = x0;
    if y0 < 0.0 {
        x -= y0 * dxdy;
        y0 = 0.0;
    }
    if y1 > height as f32 {
        y1 = height as f32;
    }
    if y0 >= y1 {
        return;
    }
    let (y_start, y_end) = (y0.floor() as i32, y1.ceil() as i32);
    for y in y_start..y_end {
        let row = y as usize * stride;
        let dy = (y as f32 + 1.0).min(y1) - (y as f32).max(y0);
        let x_next = x + dxdy * dy;
        let d = dy * dir;
        let (xa, xb) = (x.min(x_next), x.max(x_next));
        let xa_floor = xa.floor() as i32;
        let xb_ceil = xb.ceil();
        let at = |i: i32| row + i.max(0) as usize;
        if xb_ceil <= (xa_floor + 1) as f32 {
            let xmf = 0.5 * (x + x_next) - xa_floor as f32;
            acc[at(xa_floor)] += d - d * xmf;
            acc[at(xa_floor + 1)] += d * xmf;
        } else {
            let s = 1.0 / (xb - xa);
            let xaf = xa - xa_floor as f32;
            let a0 = 0.5 * s * (1.0 - xaf) * (1.0 - xaf);
            let xbf = xb - xb_ceil + 1.0;
            let am = 0.5 * s * xbf * xbf;
            let xb_int = xb_ceil as i32;
            acc[at(xa_floor)] += d * a0;
            if xb_int == xa_floor + 2 {
                acc[at(xa_floor + 1)] += d * (1.0 - a0 - am);
            } else {
                let a1 = s * (1.5 - xaf);
                acc[at(xa_floor + 1)] += d * (a1 - a0);
                for xi in xa_floor + 2..xb_int - 1 {
                    acc[at(xi)] += d * s;
                }
                let a2 = a1 + (xb_int - xa_floor - 3) as f32 * s;
                acc[at(xb_int - 1)] += d * (1.0 - a2 - am);
            }
            acc[at(xb_int)] += d * am;
        }
        x = x_next;
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn round_is_bankers() {
        assert_eq!(round(2.5), 2.0);
        assert_eq!(round(3.5), 4.0);
        assert_eq!(round(-2.5), -2.0);
        assert_eq!(round(2.4), 2.0);
    }

    #[test]
    fn numbers_as_dotnet() {
        assert_eq!(f(2.0 / 256.0), "0.007813");
        assert_eq!(f(10.24), "10.240000");
        assert_eq!(f(1.0 - 18.0 / 256.0), "0.929688");
        assert_eq!(f(0.6 / 6.0), "0.100000");
        assert_eq!(f(-0.01), "-0.010000");
        assert_eq!(f(1.0), "1.000000");
    }

    #[test]
    fn slots() {
        assert_eq!(slot_character(0x41), Some(0x41));
        assert_eq!(slot_character(0x80), Some(0x20AC));
        assert_eq!(slot_character(0x81), None);
        assert_eq!(slot_character(0x7F), None);
        assert_eq!(slot_character(0x9F), Some(0x0178));
    }

    #[test]
    fn widened_sideways_only() {
        // A 2x2 square, either winding, comes out 2.3 wide from the same left edge, as high.
        let cw = vec![vec![P { x: 1.0, y: 1.0 }, P { x: 3.0, y: 1.0 }, P { x: 3.0, y: 3.0 }, P { x: 1.0, y: 3.0 }]];
        let ccw = vec![cw[0].iter().rev().copied().collect::<Vec<P>>()];
        for square in [cw, ccw] {
            let w = embolden_x(&square, 0.3);
            let xs: Vec<f32> = w[0].iter().map(|p| p.x).collect();
            let ys: Vec<f32> = w[0].iter().map(|p| p.y).collect();
            let min = xs.iter().copied().fold(f32::MAX, f32::min);
            let max = xs.iter().copied().fold(f32::MIN, f32::max);
            assert!((min - 1.0).abs() < 1e-5 && (max - 3.3).abs() < 1e-5, "{xs:?}");
            assert_eq!(ys, square[0].iter().map(|p| p.y).collect::<Vec<f32>>());
        }
    }

    #[test]
    fn square_fills() {
        let square = vec![vec![P { x: 1.0, y: 1.0 }, P { x: 3.0, y: 1.0 }, P { x: 3.0, y: 3.0 }, P { x: 1.0, y: 3.0 }]];
        let c = fill(&square, 4, 4, 0.0, 0.0);
        assert_eq!(c[1 * 4 + 1], 255);
        assert_eq!(c[0], 0);
        assert_eq!(c[3 * 4 + 3], 0);
    }
}
