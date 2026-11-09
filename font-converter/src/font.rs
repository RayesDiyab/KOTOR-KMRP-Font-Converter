// KMRP Font Converter: reading font files.
//
// Until 2026-11-09 (the C# program) every font was read through Windows' own DirectWrite.
// The Rust program reads them itself, the same on Windows, macOS and Linux: skrifa (the
// fontations project's glyph scaler) for TrueType and OpenType of both outline kinds,
// collections (.ttc, .otc) and variable fonts (one face per named instance); WOFF is
// inflated here, as before; WOFF2 is unpacked by wuff (a Rust port of Google's woff2
// decoder). Nothing is written to a temporary folder any more: a web font is unpacked in
// memory.
//
// The metrics are DirectWrite's: the em, and the ascent and descent from OS/2's
// usWinAscent and usWinDescent, or its typo values where the font asks for them
// (USE_TYPO_METRICS), else hhea's. Outlines are y down with the origin on the baseline, as
// DirectWrite gave them, and cubic and quadratic segments are flattened by the same rule.

use std::path::{Path, PathBuf};
use std::sync::{Arc, OnceLock};

use skrifa::instance::{Location, LocationRef, Size};
use skrifa::outline::{DrawSettings, HintingInstance, HintingOptions, OutlinePen, SmoothMode, Target};
use skrifa::raw::types::{GlyphId, NameId, Tag};
use skrifa::raw::{FileRef, FontRef, TableProvider};
use skrifa::MetadataProvider;

/// A point of an outline, in pixels, y down.
#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct P {
    pub x: f32,
    pub y: f32,
}

pub type Contours = Vec<Vec<P>>;

/// A message for the player: what the converter's own checks say is wrong.
#[derive(Debug, Clone)]
pub struct FontError(pub String);

impl std::fmt::Display for FontError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(&self.0)
    }
}

impl std::error::Error for FontError {}

fn err<T>(message: &str) -> Result<T, FontError> {
    Err(FontError(message.to_string()))
}

/// One face of a font file: what the player picks.
#[derive(Clone, Debug)]
pub struct FontFaceChoice {
    pub path: PathBuf,
    pub face_index: u32,
    /// The axis settings of a variable font's named instance, or None.
    pub axes: Option<Vec<(Tag, f32)>>,
    /// As shown to the player.
    pub name: String,
}

/// A loaded face, ready to give glyph outlines. Shared between tiles and the worker.
pub struct LoadedFace {
    data: Arc<Vec<u8>>,
    index: u32,
    location: Location,
    pub units_per_em: i32,
    pub ascent: i32,
    pub descent: i32,
    pub name: String,
    /// Atlas::glyph_box, once a face.
    pub(crate) box_cache: OnceLock<(f32, f32)>,
}

impl LoadedFace {
    fn font(&self) -> FontRef<'_> {
        // Checked when the face was loaded.
        FontRef::from_index(&self.data, self.index).expect("a face that loaded")
    }

    fn loc(&self) -> LocationRef<'_> {
        LocationRef::from(&self.location)
    }

    /// The glyph for a code point, 0 when the face has none.
    pub fn glyph(&self, code_point: u32) -> u16 {
        self.font()
            .charmap()
            .map(code_point)
            .map(|g| g.to_u32().min(u16::MAX as u32) as u16)
            .unwrap_or(0)
    }

    /// The glyph's advance in design units.
    pub fn advance(&self, glyph: u16) -> i32 {
        let font = self.font();
        let metrics = font.glyph_metrics(Size::unscaled(), self.loc());
        metrics.advance_width(GlyphId::new(glyph as u32)).map(|a| a.round() as i32).unwrap_or(0)
    }

    /// The glyph's outline at `em_size` pixels, y down, origin on the baseline.
    pub fn outline(&self, glyph: u16, em_size: f32) -> Contours {
        let font = self.font();
        let outlines = font.outline_glyphs();
        let mut sink = Sink::default();
        if let Some(g) = outlines.get(GlyphId::new(glyph as u32)) {
            let _ = g.draw(DrawSettings::unhinted(Size::new(em_size), self.loc()), &mut sink);
        }
        sink.contours
    }

    /// A hinter for this face at `em_size` pixels: the font's own instructions (TrueType or
    /// PostScript hints; the automatic hinter where a TrueType font has none), for
    /// anti-aliased text (FreeType's normal target). Measured 2026-11-09 against the C#
    /// program's DirectWrite GDI-classic letters for Arial, Georgia and KMRP's Dialogue OTF at
    /// the original game's sizes, this target was the nearest of the four tried (mean alpha
    /// difference 31 to 38 over inked pixels; the strong, monochrome target 27 to 66 and
    /// jagged on Georgia; unhinted 32 to 56). None when the font's hints cannot be run.
    pub fn hinter(&self, em_size: f32) -> Option<HintingInstance> {
        let font = self.font();
        let outlines = font.outline_glyphs();
        let target = Target::from(SmoothMode::Normal);
        HintingInstance::new(&outlines, Size::new(em_size), self.loc(), HintingOptions::from(target)).ok()
    }

    /// The glyph's hinted outline (y down, origin on the baseline). Err when the hinter
    /// fails on it.
    pub fn hinted_outline(&self, glyph: u16, hinter: &HintingInstance) -> Result<Contours, FontError> {
        let font = self.font();
        let outlines = font.outline_glyphs();
        let mut sink = Sink::default();
        if let Some(g) = outlines.get(GlyphId::new(glyph as u32)) {
            g.draw(DrawSettings::hinted(hinter, false), &mut sink)
                .map_err(|_| FontError("A letter of this font could not be drawn.".into()))?;
        }
        Ok(sink.contours)
    }
}

// ---- outlines into contours ----

#[derive(Default)]
struct Sink {
    contours: Contours,
    pen: P,
}

impl Sink {
    fn current(&mut self) -> &mut Vec<P> {
        if self.contours.is_empty() {
            self.contours.push(vec![self.pen]);
        }
        self.contours.last_mut().unwrap()
    }

    // Cubic segments, flattened to lines fine enough that no step is visible at the size the
    // glyph is drawn: DirectWrite's rule, kept so the KMRP version draws as before.
    fn cubic(&mut self, p1: P, p2: P, p3: P) {
        let p0 = self.pen;
        let length = dist(p0, p1) + dist(p1, p2) + dist(p2, p3);
        let steps = ((length.sqrt() * 1.5).ceil() as i32).clamp(2, 64);
        let contour = self.current();
        for s in 1..=steps {
            let t = s as f32 / steps as f32;
            let u = 1.0 - t;
            let (a, b, c, d) = (u * u * u, 3.0 * u * u * t, 3.0 * u * t * t, t * t * t);
            contour.push(P {
                x: a * p0.x + b * p1.x + c * p2.x + d * p3.x,
                y: a * p0.y + b * p1.y + c * p2.y + d * p3.y,
            });
        }
        self.pen = p3;
    }
}

fn dist(a: P, b: P) -> f64 {
    let (dx, dy) = ((a.x - b.x) as f64, (a.y - b.y) as f64);
    (dx * dx + dy * dy).sqrt()
}

impl OutlinePen for Sink {
    fn move_to(&mut self, x: f32, y: f32) {
        self.pen = P { x, y: -y };
        self.contours.push(vec![self.pen]);
    }

    fn line_to(&mut self, x: f32, y: f32) {
        self.pen = P { x, y: -y };
        let pen = self.pen;
        self.current().push(pen);
    }

    // A quadratic segment as the cubic it is (DirectWrite gave TrueType's this way).
    fn quad_to(&mut self, cx0: f32, cy0: f32, x: f32, y: f32) {
        let p0 = self.pen;
        let c = P { x: cx0, y: -cy0 };
        let p3 = P { x, y: -y };
        let p1 = P { x: p0.x + 2.0 / 3.0 * (c.x - p0.x), y: p0.y + 2.0 / 3.0 * (c.y - p0.y) };
        let p2 = P { x: p3.x + 2.0 / 3.0 * (c.x - p3.x), y: p3.y + 2.0 / 3.0 * (c.y - p3.y) };
        self.cubic(p1, p2, p3);
    }

    fn curve_to(&mut self, cx0: f32, cy0: f32, cx1: f32, cy1: f32, x: f32, y: f32) {
        self.cubic(P { x: cx0, y: -cy0 }, P { x: cx1, y: -cy1 }, P { x, y: -y });
    }

    fn close(&mut self) {}
}

// ---- files ----

pub const EXTENSIONS: [&str; 6] = ["ttf", "otf", "ttc", "otc", "woff", "woff2"];

/// No real font comes near this; it keeps a damaged or hostile file from making the program
/// reserve gigabytes (audit F6).
pub const MAX_FONT: u64 = 128 << 20;

fn extension(path: &Path) -> String {
    path.extension().and_then(|e| e.to_str()).unwrap_or("").to_ascii_lowercase()
}

pub fn accepted(path: &Path) -> bool {
    EXTENSIONS.contains(&extension(path).as_str())
}

/// The file's bytes as a plain font: WOFF and WOFF2 unpacked.
fn read_font_file(path: &Path) -> Result<Arc<Vec<u8>>, FontError> {
    let meta = std::fs::metadata(path).map_err(|e| io_error(&e))?;
    if meta.len() > MAX_FONT {
        return err("This font file is too large.");
    }
    let data = std::fs::read(path).map_err(|e| io_error(&e))?;
    let font = match extension(path).as_str() {
        "woff" => woff(&data)?,
        "woff2" => {
            let font = wuff::decompress_woff2(&data)
                .map_err(|_| FontError("This web font could not be unpacked. Pick the font's TTF or OTF instead.".into()))?;
            if font.len() as u64 > MAX_FONT {
                return err("This web font unpacks to more than a font can be.");
            }
            font
        }
        _ => data,
    };
    Ok(Arc::new(font))
}

/// The converter's own words for a file that cannot be read.
pub fn io_error(e: &std::io::Error) -> FontError {
    FontError(match e.kind() {
        std::io::ErrorKind::NotFound => "The file is not there any more.".into(),
        std::io::ErrorKind::PermissionDenied => "The system did not allow the file to be read.".into(),
        _ => format!("The file could not be read as a font. ({e})"),
    })
}

const NOT_A_FONT: &str = "This file is not a font that can be read.";

/// Every face the player can pick from a file: one per face of a collection, and one per
/// named instance of a variable font. Err with a message for the player when the file is
/// not a font that can be read.
pub fn faces(path: &Path) -> Result<Vec<FontFaceChoice>, FontError> {
    let data = read_font_file(path)?;
    let file = FileRef::new(&data).map_err(|_| FontError(NOT_A_FONT.into()))?;
    let count = file.fonts().count();
    if count == 0 {
        return err(NOT_A_FONT);
    }
    let mut choices = Vec::new();
    for i in 0..count as u32 {
        let face = load_from(data.clone(), &FontFaceChoice {
            path: path.to_path_buf(),
            face_index: i,
            axes: None,
            name: String::new(),
        })?;
        let instances = named_instances(&face);
        if instances.is_empty() {
            choices.push(FontFaceChoice { path: path.to_path_buf(), face_index: i, axes: None, name: face.name.clone() });
        }
        for (name, axes) in instances {
            choices.push(FontFaceChoice { path: path.to_path_buf(), face_index: i, axes: Some(axes), name });
        }
    }
    Ok(choices)
}

pub fn load(choice: &FontFaceChoice) -> Result<Arc<LoadedFace>, FontError> {
    let data = read_font_file(&choice.path)?;
    load_from(data, choice).map(Arc::new)
}

fn load_from(data: Arc<Vec<u8>>, choice: &FontFaceChoice) -> Result<LoadedFace, FontError> {
    let (units_per_em, ascent, descent, location, name) = {
        let font = FontRef::from_index(&data, choice.face_index).map_err(|_| FontError(NOT_A_FONT.into()))?;
        let head = font.head().map_err(|_| FontError(NOT_A_FONT.into()))?;
        let units_per_em = head.units_per_em() as i32;
        if units_per_em <= 0 {
            return err(NOT_A_FONT);
        }
        // Outlines are what the converter draws from; a font of bitmaps or colour layers
        // only has none (DirectWrite refused those too).
        if font.outline_glyphs().format().is_none() {
            return err(NOT_A_FONT);
        }
        let (ascent, descent) = vertical_metrics(&font);
        let location = match &choice.axes {
            Some(axes) => font.axes().location(axes.iter().map(|(t, v)| (*t, *v))),
            None => Location::default(),
        };
        let name = if choice.name.is_empty() {
            full_name(&font).unwrap_or_else(|| {
                choice.path.file_stem().map(|s| s.to_string_lossy().into_owned()).unwrap_or_else(|| "Font".into())
            })
        } else {
            choice.name.clone()
        };
        (units_per_em, ascent, descent, location, name)
    };
    Ok(LoadedFace {
        data,
        index: choice.face_index,
        location,
        units_per_em,
        ascent,
        descent,
        name,
        box_cache: OnceLock::new(),
    })
}

/// DirectWrite's ascent and descent: OS/2's Windows values, or its typo values where the font
/// asks for them, else hhea's.
fn vertical_metrics(font: &FontRef) -> (i32, i32) {
    if let Ok(os2) = font.os2() {
        if os2.fs_selection().bits() & 0x0080 != 0 {
            return (os2.s_typo_ascender() as i32, -(os2.s_typo_descender() as i32));
        }
        if os2.us_win_ascent() != 0 || os2.us_win_descent() != 0 {
            return (os2.us_win_ascent() as i32, os2.us_win_descent() as i32);
        }
    }
    if let Ok(hhea) = font.hhea() {
        return (hhea.ascender().to_i16() as i32, -(hhea.descender().to_i16() as i32));
    }
    (0, 0)
}

/// A name from the 'name' table, English Windows first: Windows US English, then any Windows,
/// then Unicode, then Mac Roman.
fn name_record(font: &FontRef, id: u16) -> Option<String> {
    let name = font.name().ok()?;
    let data = name.string_data();
    let mut best: Option<String> = None;
    let mut best_score = -1;
    for record in name.name_record() {
        if record.name_id() != NameId::new(id) {
            continue;
        }
        let (platform, encoding, language) = (record.platform_id(), record.encoding_id(), record.language_id());
        let score = if platform == 3 && language == 0x409 {
            3
        } else if platform == 3 {
            2
        } else if platform == 0 {
            1
        } else if platform == 1 && encoding == 0 {
            0
        } else {
            -1
        };
        if score <= best_score {
            continue;
        }
        let Ok(text) = record.string(data) else { continue };
        best = Some(text.chars().collect::<String>().trim().to_string());
        best_score = score;
    }
    best.filter(|s| !s.is_empty())
}

fn full_name(font: &FontRef) -> Option<String> {
    name_record(font, 4).or_else(|| name_record(font, 1))
}

/// A variable font's named instances ("Light", "Bold"...), as names and axis settings.
fn named_instances(face: &LoadedFace) -> Vec<(String, Vec<(Tag, f32)>)> {
    let font = face.font();
    let axes: Vec<Tag> = font.axes().iter().map(|a| a.tag()).collect();
    if axes.is_empty() {
        return Vec::new();
    }
    let family = name_record(&font, 16).or_else(|| name_record(&font, 1)).unwrap_or_else(|| "Font".into());
    let mut seen = std::collections::HashSet::new();
    let mut list = Vec::new();
    for (n, instance) in font.named_instances().iter().enumerate() {
        let sub = name_record(&font, instance.subfamily_name_id().to_u16()).unwrap_or_else(|| format!("Style {}", n + 1));
        let values: Vec<(Tag, f32)> = axes.iter().copied().zip(instance.user_coords()).collect();
        let name = format!("{family} {sub}");
        if seen.insert(name.clone()) {
            list.push((name, values));
        }
    }
    list
}

// ---- WOFF 1.0 ----

fn u16_at(b: &[u8], o: usize) -> u16 {
    (b[o] as u16) << 8 | b[o + 1] as u16
}

fn u32_at(b: &[u8], o: usize) -> u32 {
    (b[o] as u32) << 24 | (b[o + 1] as u32) << 16 | (b[o + 2] as u32) << 8 | b[o + 3] as u32
}

/// A WOFF 1.0 file back to the font it packs: each table inflated (zlib) and laid out again
/// behind a plain table directory. The limits are the C# program's (audit F6).
pub fn woff(w: &[u8]) -> Result<Vec<u8>, FontError> {
    const DAMAGED: &str = "This WOFF font is damaged.";
    if w.len() < 44 || u32_at(w, 0) != 0x774F_4646 {
        return err("This is not a WOFF font.");
    }
    let flavor = u32_at(w, 4);
    let tables = u16_at(w, 12) as usize;
    let total = u32_at(w, 16) as u64; // totalSfntSize: the font's size unpacked, as the file says
    if tables == 0 || 44 + tables as u64 * 20 > w.len() as u64 || total > MAX_FONT {
        return err(DAMAGED);
    }
    let mut tags = Vec::with_capacity(tables);
    let mut sums = Vec::with_capacity(tables);
    let mut bodies: Vec<Vec<u8>> = Vec::with_capacity(tables);
    let mut unpacked: u64 = 12 + 16 * tables as u64;
    for i in 0..tables {
        let r = 44 + i * 20;
        tags.push(u32_at(w, r));
        let offset = u32_at(w, r + 4) as u64;
        let compressed = u32_at(w, r + 8) as u64;
        let length = u32_at(w, r + 12) as u64;
        sums.push(u32_at(w, r + 16));
        unpacked += (length + 3) & !3;
        if offset + compressed > w.len() as u64 || compressed > length || unpacked > total + 3 || unpacked > MAX_FONT {
            return err(DAMAGED);
        }
        let (offset, compressed, length) = (offset as usize, compressed as usize, length as usize);
        let body = if compressed == length {
            w[offset..offset + length].to_vec()
        } else {
            let body = miniz_oxide::inflate::decompress_to_vec_zlib_with_limit(&w[offset..offset + compressed], length)
                .map_err(|_| FontError(DAMAGED.into()))?;
            if body.len() != length {
                return err(DAMAGED);
            }
            body
        };
        bodies.push(body);
    }
    let mut head = vec![0u8; 12 + 16 * tables];
    put32(&mut head, 0, flavor);
    let (mut power, mut log) = (1usize, 0usize);
    while power * 2 <= tables {
        power *= 2;
        log += 1;
    }
    put16(&mut head, 4, tables as u16);
    put16(&mut head, 6, (power * 16) as u16);
    put16(&mut head, 8, log as u16);
    put16(&mut head, 10, (tables * 16 - power * 16) as u16);
    let mut at = head.len();
    for i in 0..tables {
        let r = 12 + i * 16;
        put32(&mut head, r, tags[i]);
        put32(&mut head, r + 4, sums[i]);
        put32(&mut head, r + 8, at as u32);
        put32(&mut head, r + 12, bodies[i].len() as u32);
        at += (bodies[i].len() + 3) & !3;
    }
    let mut out = head;
    for body in &bodies {
        out.extend_from_slice(body);
        out.resize(out.len() + (((body.len() + 3) & !3) - body.len()), 0);
    }
    Ok(out)
}

fn put16(b: &mut [u8], o: usize, v: u16) {
    b[o] = (v >> 8) as u8;
    b[o + 1] = v as u8;
}

fn put32(b: &mut [u8], o: usize, v: u32) {
    b[o..o + 4].copy_from_slice(&v.to_be_bytes());
}
