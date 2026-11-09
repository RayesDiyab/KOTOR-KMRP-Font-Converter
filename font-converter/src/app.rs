// KMRP Font Converter: the window.
//
// One window, painted by hand in the KMRP installer's colours: eleven tiles, one for each
// distinct font the game draws its text in, grouped as the game uses them. A tile shows its
// text in KMRP's Original HD Font until the player picks a font for it, then in that font.
// Convert writes the picked fonts' atlases (conversion.rs, atlas.rs) into a folder, for the
// version chosen in the Version box beside it: KMRP, which sizes them to the screen, or the
// original game, at the game's own font sizes. With the original game chosen the logo's word
// is KOTOR.
//
// Everything is laid out on a 1600x900 canvas and scaled to the window, so the layout is the
// approved mockup's at any size and any DPI; pictures are made at the screen's own pixels
// and put on whole pixels. Pop-ups are drawn in the same window.
//
// Ported on 2026-11-09 from MainForm.cs, Dialogs.cs and Theme.cs (WinForms and GDI+) to
// egui, which draws the same on Windows, macOS and Linux. The layout's numbers are the C#
// program's.

use std::collections::HashMap;
use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicBool, AtomicI32, AtomicUsize, Ordering};
use std::sync::{Arc, Mutex};

use eframe::egui::{
    self, pos2, vec2, Color32, CursorIcon, Key, Mesh, Pos2, Rect, Shape, Stroke, StrokeKind, TextureHandle,
    TextureOptions, ViewportCommand,
};

use crate::atlas::{self, Reference, Target};
use crate::conversion::{self, Conversion, Failure};
use crate::font::{self, FontFaceChoice, LoadedFace};
use crate::places;
use crate::theme::{self, alpha, body, display, mono, Font, Theme, ACCENT_BOTTOM, ACCENT_HOVER, ACCENT_INK, ACCENT_TOP};
use crate::tiles::{self, Piece, Tile, TileState};
use crate::Args;

const CW: f32 = 1600.0;
const CH: f32 = 900.0;

// ---- what the command line and the window share ----

pub fn not_a_font_type(path: &Path) -> String {
    format!(
        "Pick a TTF, OTF, TTC, OTC, WOFF or WOFF2 font file. {} is not one.",
        path.file_name().map(|n| n.to_string_lossy().into_owned()).unwrap_or_default()
    )
}

pub fn write_failure(message: &str, moving: bool) -> String {
    if moving {
        format!("{message} Some fonts may already be in the folder: convert again to finish.")
    } else if message.contains("Nothing was changed") {
        message.to_string()
    } else {
        format!("{message} Nothing in the folder was changed.")
    }
}

/// A face loaded and checked as Apply checked it: a font without the Latin letters would make
/// the game's text invisible (audit F5). The face and the game's letters it lacks, or a
/// message's title and body.
pub fn checked_face(choice: &FontFaceChoice) -> Result<(Arc<LoadedFace>, Vec<char>), (String, String)> {
    let face = font::load(choice).map_err(|e| ("This font can't be read".to_string(), e.0))?;
    let missing = atlas::missing_basic_latin(&face);
    if !missing.is_empty() {
        let body = if missing.chars().count() >= 52 {
            "It has no letters A to Z, so the game's text would be invisible. Pick a font made for Latin letters.".to_string()
        } else {
            let list: Vec<String> = missing.chars().map(|c| c.to_string()).collect();
            format!(
                "It has no {}, so those letters would be invisible in the game. Pick a font with every letter from A to Z.",
                list.join(", ")
            )
        };
        return Err(("This font can't be used".to_string(), body));
    }
    let all = atlas::missing(&face);
    Ok((face, all))
}

// ---- pictures ----

/// A picture in premultiplied RGBA.
#[derive(Clone)]
struct Img {
    w: usize,
    h: usize,
    rgba: Vec<u8>,
}

fn decode(png: &[u8]) -> Option<Img> {
    let img = image::load_from_memory_with_format(png, image::ImageFormat::Png).ok()?.to_rgba8();
    let (w, h) = (img.width() as usize, img.height() as usize);
    let mut rgba = img.into_raw();
    for px in rgba.chunks_exact_mut(4) {
        let a = px[3] as u32;
        for c in &mut px[..3] {
            *c = ((*c as u32 * a + 127) / 255) as u8;
        }
    }
    Some(Img { w, h, rgba })
}

/// The picture at `w` x `h`, each new pixel the average of the old ones it covers (a box
/// filter, as good as GDI+'s bicubic for shrinking).
fn resample(src: &Img, w: usize, h: usize) -> Vec<u8> {
    let mut out = vec![0u8; w * h * 4];
    let (sx, sy) = (src.w as f32 / w as f32, src.h as f32 / h as f32);
    for y in 0..h {
        let (y0, y1) = (y as f32 * sy, ((y + 1) as f32 * sy).min(src.h as f32));
        for x in 0..w {
            let (x0, x1) = (x as f32 * sx, ((x + 1) as f32 * sx).min(src.w as f32));
            let mut acc = [0f32; 4];
            let mut area = 0f32;
            let mut yy = y0.floor() as usize;
            while (yy as f32) < y1 && yy < src.h {
                let wy = (y1.min(yy as f32 + 1.0) - y0.max(yy as f32)).max(0.0);
                let mut xx = x0.floor() as usize;
                while (xx as f32) < x1 && xx < src.w {
                    let wx = (x1.min(xx as f32 + 1.0) - x0.max(xx as f32)).max(0.0);
                    let weight = wx * wy;
                    let o = (yy * src.w + xx) * 4;
                    for c in 0..4 {
                        acc[c] += src.rgba[o + c] as f32 * weight;
                    }
                    area += weight;
                    xx += 1;
                }
                yy += 1;
            }
            let o = (y * w + x) * 4;
            for c in 0..4 {
                out[o + c] = if area > 0.0 { (acc[c] / area).round().min(255.0) as u8 } else { 0 };
            }
        }
    }
    out
}

/// The opaque part of a picture, with two pixels round it.
fn crop_ink(img: &Img) -> Img {
    let (mut left, mut top, mut right, mut bottom) = (img.w, img.h, 0usize, 0usize);
    let mut any = false;
    for y in (0..img.h).step_by(2) {
        for x in (0..img.w).step_by(2) {
            if img.rgba[(y * img.w + x) * 4 + 3] > 8 {
                any = true;
                left = left.min(x);
                right = right.max(x);
                top = top.min(y);
                bottom = bottom.max(y);
            }
        }
    }
    if !any {
        return img.clone();
    }
    let (l, t) = (left.saturating_sub(2), top.saturating_sub(2));
    let (r, b) = ((right + 3).min(img.w), (bottom + 3).min(img.h));
    let (w, h) = (r - l, b - t);
    let mut rgba = vec![0u8; w * h * 4];
    for y in 0..h {
        let s = ((t + y) * img.w + l) * 4;
        rgba[y * w * 4..(y + 1) * w * 4].copy_from_slice(&img.rgba[s..s + w * 4]);
    }
    Img { w, h, rgba }
}

/// The window's icon: the sigil above FONT on a transparent square.
fn icon() -> Option<egui::IconData> {
    let logo = decode(crate::FONT_LOGO)?;
    let side = logo.w.max(logo.h);
    let mut square = Img { w: side, h: side, rgba: vec![0u8; side * side * 4] };
    let (ox, oy) = ((side - logo.w) / 2, (side - logo.h) / 2);
    for y in 0..logo.h {
        let d = ((oy + y) * side + ox) * 4;
        square.rgba[d..d + logo.w * 4].copy_from_slice(&logo.rgba[y * logo.w * 4..(y + 1) * logo.w * 4]);
    }
    let mut rgba = resample(&square, 256, 256);
    // Back to straight alpha, which the window system wants.
    for px in rgba.chunks_exact_mut(4) {
        let a = px[3] as u32;
        if a > 0 {
            for c in &mut px[..3] {
                *c = ((*c as u32 * 255 + a / 2) / a).min(255) as u8;
            }
        }
    }
    Some(egui::IconData { rgba, width: 256, height: 256 })
}

/// A texture made at the screen's pixels, with the size it was made at.
struct Tex {
    handle: TextureHandle,
    w: usize,
    h: usize,
    /// For a line of text: the glyph box asked for, and the margin round it.
    box_h: i32,
    pad: i32,
}

// ---- input ----

#[derive(Clone, Copy, PartialEq)]
enum Act {
    Settings,
    Choose(usize),
    Clear(usize),
    PickFolder,
    ClearAll,
    SetTarget(Target),
    Convert,
    OpenOutput,
    Button(usize),
    Row(usize),
    TextSize(usize),
    Light(bool),
}

struct Hit {
    rect: Rect,
    id: String,
    act: Act,
}

// ---- pop-ups ----

#[derive(Clone, Copy)]
enum DAct {
    Close,
    Replace,
    PickerUse,
    PickerOther,
}

struct Picker {
    tile: usize,
    faces: Vec<FontFaceChoice>,
    selected: usize,
    top: usize,
    reopen: bool,
    sample: String,
    loaded: HashMap<usize, Option<Arc<LoadedFace>>>,
    samples: HashMap<usize, (String, Option<Tex>)>,
}

enum Kind {
    Message,
    Settings,
    Picker(Box<Picker>),
}

struct Dialog {
    kind: Kind,
    title: String,
    body: String,
    width: f32,
    buttons: Vec<(String, bool, DAct)>,
}

const VISIBLE: usize = 6;

// ---- the conversion running ----

struct Job {
    progress: Arc<AtomicUsize>,
    total: usize,
    stop: Arc<AtomicBool>,
    result: Arc<Mutex<Option<(Result<(), Failure>, bool)>>>,
    tiles: Vec<usize>,
}

pub fn run(args: Args) -> i32 {
    let exit_code = Arc::new(AtomicI32::new(0));
    let code = exit_code.clone();
    let mut viewport = egui::ViewportBuilder::default()
        .with_title("KMRP Font Converter")
        .with_app_id("kmrp-font-converter")
        .with_inner_size([1280.0, 720.0])
        .with_min_inner_size([640.0, 360.0])
        .with_drag_and_drop(true);
    if let Some(icon) = icon() {
        viewport = viewport.with_icon(Arc::new(icon));
    }
    let options = eframe::NativeOptions { viewport, centered: true, ..Default::default() };
    let result = eframe::run_native(
        "KMRP Font Converter",
        options,
        Box::new(move |cc| {
            theme::install_fonts(&cc.egui_ctx);
            Ok(Box::new(App::new(cc.egui_ctx.clone(), args, code)))
        }),
    );
    if let Err(e) = result {
        eprintln!("The window could not be opened: {e}");
        return 1;
    }
    exit_code.load(Ordering::SeqCst)
}

pub struct App {
    ctx: egui::Context,
    theme: Theme,
    tiles: Vec<Tile>,
    references: Arc<HashMap<String, Reference>>,
    kmrp_logo: Option<Img>,
    kotor_logo: Option<Img>,
    gear: Option<Img>,
    examples: HashMap<String, Img>,
    scaled: HashMap<String, Tex>,
    previews: HashMap<(usize, String), Tex>,
    measures: HashMap<(usize, String), (i32, Option<(usize, i32)>)>,
    output: PathBuf,
    target: Target,

    scale: f32,
    ox: f32,
    oy: f32,
    ppp: f32,
    view: Rect,
    hits: Vec<Hit>,
    hover: Option<String>,
    pressed: Option<String>,
    dialog: Option<Dialog>,
    drop_target: Option<usize>,
    pending_drop: Option<PathBuf>,
    job: Option<Job>,
    still: bool,
    close_when_done: bool,
    converted: bool,
    scroll: f32,
    first_frame: bool,
    size_arg: Option<(u32, u32)>,
    convert_on_show: bool,
    exit_code: Arc<AtomicI32>,
    last_font_folder: Option<PathBuf>,
    monitor: Option<egui::Vec2>,
    /// The missing-letters badge pointed at: its place and its tile.
    tooltip: Option<(Rect, usize)>,
}

impl App {
    fn new(ctx: egui::Context, args: Args, exit_code: Arc<AtomicI32>) -> App {
        let (size, light) = places::load_look();
        let mut theme = Theme { light, text_scale: places::TEXT_SCALES[size] };
        if let Some(s) = args.text_size {
            theme.text_scale = places::TEXT_SCALES[s];
        }
        if let Some(l) = args.light {
            theme.light = l;
        }
        let output = args.out.clone().unwrap_or_else(|| places::saved_folder().unwrap_or_else(places::default_folder));
        let target = args.version.unwrap_or_else(|| places::load_target(&output));
        let examples = crate::EXAMPLE_PNGS.iter().filter_map(|(n, png)| decode(png).map(|i| (n.to_string(), i))).collect();
        let mut app = App {
            ctx,
            theme,
            tiles: tiles::all(crate::EXAMPLES),
            references: Arc::new(atlas::references(crate::REFERENCES, crate::ORIGINALS)),
            kmrp_logo: decode(crate::KMRP_LOGO),
            kotor_logo: decode(crate::KOTOR_LOGO),
            gear: decode(crate::GEAR).map(|g| crop_ink(&g)),
            examples,
            scaled: HashMap::new(),
            previews: HashMap::new(),
            measures: HashMap::new(),
            output,
            target,
            scale: 1.0,
            ox: 0.0,
            oy: 0.0,
            ppp: 1.0,
            view: Rect::ZERO,
            hits: Vec::new(),
            hover: None,
            pressed: None,
            dialog: None,
            drop_target: None,
            pending_drop: None,
            job: None,
            still: args.still,
            close_when_done: false,
            converted: false,
            scroll: 0.0,
            first_frame: true,
            size_arg: args.size,
            convert_on_show: args.convert,
            exit_code,
            last_font_folder: None,
            monitor: None,
            tooltip: None,
        };
        app.ctx.send_viewport_cmd(ViewportCommand::SetTheme(if app.theme.light {
            egui::SystemTheme::Light
        } else {
            egui::SystemTheme::Dark
        }));
        if args.settings {
            app.dialog = Some(App::settings_dialog());
        }
        for spec in &args.fonts {
            let Some(tile) = app.tiles.iter().position(|t| t.def.key == spec.key) else { continue };
            // As if the player had chosen the file: the same checks and messages.
            if spec.pick {
                app.open(tile, &spec.path);
                continue;
            }
            let faces = match font::faces(&spec.path) {
                Ok(f) => f,
                Err(e) => {
                    app.message("This font can't be read", &e.0);
                    continue;
                }
            };
            let choice = faces.iter().find(|f| spec.face.as_ref().is_none_or(|n| &f.name == n)).unwrap_or(&faces[0]).clone();
            app.apply(tile, choice, faces);
        }
        app
    }

    fn busy(&self) -> bool {
        self.job.is_some()
    }

    // ---- canvas to screen ----

    fn px(&self, x: f32) -> f32 {
        self.ox + x * self.scale
    }

    fn py(&self, y: f32) -> f32 {
        self.oy + y * self.scale
    }

    fn pt(&self, x: f32, y: f32) -> Pos2 {
        pos2(self.px(x), self.py(y))
    }

    fn rect(&self, r: Rect) -> Rect {
        Rect::from_min_max(self.pt(r.min.x, r.min.y), self.pt(r.max.x, r.max.y))
    }

    /// A length in points put on the screen's pixel grid.
    fn snap(&self, v: f32) -> f32 {
        (v * self.ppp).round() / self.ppp
    }

    /// The canvas's scale in the screen's own pixels.
    fn device(&self) -> f32 {
        self.scale * self.ppp
    }

    fn add_hit(&mut self, rect: Rect, id: impl Into<String>, act: Act) {
        self.hits.push(Hit { rect, id: id.into(), act });
    }

    fn is_hover(&self, id: &str) -> bool {
        self.hover.as_deref() == Some(id)
    }

    // ---- text ----

    fn font_id(&self, font: Font) -> egui::FontId {
        font.id(font.px * self.theme.text_scale * self.scale)
    }

    fn galley(&self, p: &egui::Painter, s: &str, font: Font, colour: Color32) -> Arc<egui::Galley> {
        p.layout_no_wrap(s.to_string(), self.font_id(font), colour)
    }

    /// A line's width on the canvas.
    fn width(&self, p: &egui::Painter, s: &str, font: Font) -> f32 {
        self.galley(p, s, font, Color32::WHITE).size().x / self.scale
    }

    /// Text whose line starts at canvas point (x, y), on whole pixels.
    fn draw_at(&self, p: &egui::Painter, s: &str, font: Font, colour: Color32, x: f32, y: f32) {
        let g = self.galley(p, s, font, colour);
        p.galley(pos2(self.snap(self.px(x)), self.snap(self.py(y))), g, colour);
    }

    /// The text cut to `width` with an ellipsis: at its end, or in its middle for a path.
    fn fit(&self, p: &egui::Painter, s: &str, font: Font, width: f32, path: bool) -> String {
        if self.width(p, s, font) <= width + 0.5 {
            return s.to_string();
        }
        let chars: Vec<char> = s.chars().collect();
        let (mut lo, mut hi) = (0usize, chars.len());
        let make = |n: usize| -> String {
            if path {
                let head = n / 3;
                let tail = n - head;
                let a: String = chars[..head].iter().collect();
                let b: String = chars[chars.len() - tail..].iter().collect();
                format!("{a}\u{2026}{b}")
            } else {
                let a: String = chars[..n].iter().collect();
                format!("{}\u{2026}", a.trim_end())
            }
        };
        while lo < hi {
            let mid = (lo + hi).div_ceil(2);
            if self.width(p, &make(mid), font) <= width {
                lo = mid;
            } else {
                hi = mid - 1;
            }
        }
        make(lo)
    }

    /// One line of text in `r` (canvas), centred on its height, at its left or its middle,
    /// cut with an ellipsis where it does not fit.
    fn draw(&self, p: &egui::Painter, s: &str, font: Font, colour: Color32, r: Rect, centre: bool, path: bool) {
        let s = self.fit(p, s, font, r.width(), path);
        let g = self.galley(p, &s, font, colour);
        let (w, h) = (g.size().x, g.size().y);
        let rr = self.rect(r);
        let x = if centre && w < rr.width() { rr.min.x + (rr.width() - w) / 2.0 } else { rr.min.x };
        let y = rr.min.y + (rr.height() - h) / 2.0;
        p.galley(pos2(self.snap(x), self.snap(y)), g, colour);
    }

    fn wrapped_galley(&self, p: &egui::Painter, s: &str, font: Font, colour: Color32, width: f32) -> Arc<egui::Galley> {
        p.layout(s.to_string(), self.font_id(font), colour, width * self.scale)
    }

    /// Wrapped text in `r` (canvas), its first line on a whole pixel.
    fn draw_wrapped(&self, p: &egui::Painter, s: &str, font: Font, colour: Color32, r: Rect) {
        let g = self.wrapped_galley(p, s, font, colour, r.width());
        p.galley(pos2(self.snap(self.px(r.min.x)), self.snap(self.py(r.min.y))), g, colour);
    }

    // ---- shapes ----

    fn rounded_points(&self, r: Rect, radius: f32) -> Vec<Pos2> {
        let r = self.rect(r);
        let rad = (radius * self.scale).min(r.width() / 2.0).min(r.height() / 2.0).max(0.0);
        let mut pts = Vec::new();
        let corners = [
            (r.max.x - rad, r.min.y + rad, -90f32),
            (r.max.x - rad, r.max.y - rad, 0.0),
            (r.min.x + rad, r.max.y - rad, 90.0),
            (r.min.x + rad, r.min.y + rad, 180.0),
        ];
        for (cx, cy, start) in corners {
            for k in 0..=8 {
                let a = (start + k as f32 * 90.0 / 8.0).to_radians();
                pts.push(pos2(cx + rad * a.cos(), cy + rad * a.sin()));
            }
        }
        pts
    }

    fn rbox(&self, p: &egui::Painter, r: Rect, radius: f32, fill: Color32, edge: Color32, width: f32) {
        let stroke = if edge.a() > 0 { Stroke::new(width * self.scale, edge) } else { Stroke::NONE };
        p.rect(self.rect(r), radius * self.scale, fill, stroke, StrokeKind::Middle);
    }

    /// A rounded box filled with a vertical gradient, cut at `cut_x` (canvas) when given.
    fn gradient(&self, p: &egui::Painter, r: Rect, radius: f32, top: Color32, bottom: Color32, cut_x: Option<f32>) {
        let mut pts = self.rounded_points(r, radius);
        if let Some(cut) = cut_x {
            pts = clip_right(&pts, self.px(cut));
        }
        let rr = self.rect(r);
        fill_gradient(p, &pts, rr.min.y, rr.max.y, top, bottom);
    }

    // ---- pictures at the screen's pixels ----

    fn texture(&self, name: &str, w: usize, h: usize, rgba: &[u8], nearest: bool) -> TextureHandle {
        let image = egui::ColorImage::from_rgba_premultiplied([w, h], rgba);
        self.ctx.load_texture(name, image, if nearest { TextureOptions::NEAREST } else { TextureOptions::LINEAR })
    }

    fn draw_texture(&self, p: &egui::Painter, tex: &Tex, x: f32, y: f32) {
        let at = pos2(self.snap(x), self.snap(y));
        let r = Rect::from_min_size(at, vec2(tex.w as f32 / self.ppp, tex.h as f32 / self.ppp));
        p.image(tex.handle.id(), r, Rect::from_min_max(pos2(0.0, 0.0), pos2(1.0, 1.0)), Color32::WHITE);
    }

    /// An image drawn at the screen's own pixels: scaled once into a cache, made again when
    /// its size changes. `tint` draws its alpha in one colour.
    fn draw_sharp(&mut self, p: &egui::Painter, which: &str, name: &str, r: Rect, tint: Option<Color32>) {
        let w = ((r.width() * self.device()).round() as usize).max(1);
        let h = ((r.height() * self.device()).round() as usize).max(1);
        let key = match tint {
            Some(c) => format!("{name}:{:?}", c.to_array()),
            None => name.to_string(),
        };
        let fresh = self.scaled.get(&key).is_some_and(|t| t.w == w && t.h == h);
        if !fresh {
            let src = match which {
                "kmrp" => self.kmrp_logo.as_ref(),
                "kotor" => self.kotor_logo.as_ref(),
                "gear" => self.gear.as_ref(),
                other => self.examples.get(other),
            };
            let Some(src) = src else { return };
            let mut rgba = resample(src, w, h);
            if let Some(c) = tint {
                for px in rgba.chunks_exact_mut(4) {
                    let a = px[3] as u32 * c.a() as u32 / 255;
                    px[0] = (c.r() as u32 * a / 255) as u8;
                    px[1] = (c.g() as u32 * a / 255) as u8;
                    px[2] = (c.b() as u32 * a / 255) as u8;
                    px[3] = a as u8;
                }
            }
            let handle = self.texture(&key, w, h, &rgba, true);
            self.scaled.insert(key.clone(), Tex { handle, w, h, box_h: 0, pad: 0 });
        }
        let tex = &self.scaled[&key];
        self.draw_texture(p, tex, self.px(r.min.x), self.py(r.min.y));
    }

    // ---- painting ----

    fn paint(&mut self, p: &egui::Painter) {
        self.hits.clear();
        p.rect_filled(self.view, 0.0, self.theme.window());
        self.paint_header(p);
        self.paint_groups(p);
        self.paint_actions(p);
        if let (Some((anchor, t)), None) = (self.tooltip.take(), &self.dialog) {
            self.paint_tooltip(p, anchor, t);
        }
        if self.dialog.is_some() {
            self.hits.clear();
            p.rect_filled(self.view, 0.0, self.theme.dim());
            self.paint_dialog(p);
        }
    }

    fn paint_header(&mut self, p: &egui::Painter) {
        let th = self.theme;
        // From the window's own top and sides, where the canvas does not fill it.
        let edge = (self.view.min.y - self.oy) / self.scale;
        let left = (self.view.min.x - self.ox) / self.scale;
        let right = (self.view.max.x - self.ox) / self.scale;
        {
            let (glow, window) = (th.header_glow(), th.window());
            let area = Rect::from_min_max(pos2(left, edge), pos2(right, 218.0));
            let (nx, ny) = (48usize, 12usize);
            let mut mesh = Mesh::default();
            for j in 0..=ny {
                for i in 0..=nx {
                    let x = area.min.x + area.width() * i as f32 / nx as f32;
                    let y = area.min.y + area.height() * j as f32 / ny as f32;
                    // The ellipse's glow, from its centre at the window's top to its edge.
                    let (dx, dy) = ((x - 288.0) / 900.0, (y - edge) / 260.0);
                    let d = (dx * dx + dy * dy).sqrt().min(1.0);
                    let mut c = lerp(glow, window, d);
                    // Faded into the window below y 86.
                    if y > 87.0 {
                        c = lerp(c, window, ((y - 87.0) / 131.0).clamp(0.0, 1.0));
                    }
                    mesh.colored_vertex(self.pt(x, y), c);
                }
            }
            for j in 0..ny as u32 {
                for i in 0..nx as u32 {
                    let a = j * (nx as u32 + 1) + i;
                    let b = a + nx as u32 + 1;
                    mesh.add_triangle(a, a + 1, b);
                    mesh.add_triangle(a + 1, b + 1, b);
                }
            }
            p.add(Shape::mesh(mesh));
        }

        // The logo: KMRP's, or the same lockup saying KOTOR when the fonts are made for the
        // original game. Both images are made at one scale, so their letters match.
        let kotor = self.target == Target::Original && self.kotor_logo.is_some();
        let logo = if kotor { self.kotor_logo.as_ref() } else { self.kmrp_logo.as_ref() };
        let (logo_top, logo_h) = (14.0, 116.0);
        let mut logo_w = 0.0;
        if let Some(l) = logo {
            let drawn_h = logo_h * l.h as f32 / 300.0;
            logo_w = drawn_h * l.w as f32 / l.h as f32;
            let (which, name) = if kotor { ("kotor", "logo:kotor") } else { ("kmrp", "logo:kmrp") };
            self.draw_sharp(p, which, name, Rect::from_min_size(pos2(56.0, logo_top), vec2(logo_w, drawn_h)), None);
        }
        // The title's capitals centred on the logo's middle.
        let title = display(42.0);
        let middle = logo_top + logo_h / 2.0 + 1.0;
        let title_x = 56.0 + logo_w + 26.0;
        {
            let g = self.galley(p, "Font Converter", title, th.text());
            let size = self.font_id(title).size;
            let ascent = g.rows.first().and_then(|r| r.row.glyphs.first()).map(|gl| gl.font_ascent).unwrap_or(size * 0.8);
            let cap = 0.70 * size;
            let baseline = self.py(middle) + cap / 2.0;
            let top = baseline - ascent;
            let x = self.px(title_x);
            if !th.light {
                let shadow = self.galley(p, "Font Converter", title, Color32::from_rgba_unmultiplied(0, 0, 0, 120));
                p.galley(pos2(self.snap(x + self.scale), self.snap(top + 2.0 * self.scale)), shadow, Color32::BLACK);
            }
            p.galley(pos2(self.snap(x), self.snap(top)), g, th.text());
        }

        // Settings, top right: the installer's square gear button.
        let settings = Rect::from_min_size(pos2(1488.0, middle - 26.0), vec2(52.0, 52.0));
        let on = self.is_hover("settings");
        self.rbox(p, settings, 6.0, if on { th.button_hover() } else { th.button() }, if on { th.accent() } else { th.edge() }, 1.0);
        if let Some(g) = &self.gear {
            let inner = Rect::from_min_size(pos2(settings.min.x + 9.0, settings.min.y + 9.0), vec2(34.0, 34.0));
            let fit = (inner.width() / g.w as f32).min(inner.height() / g.h as f32);
            let (w, h) = (g.w as f32 * fit, g.h as f32 * fit);
            let r = Rect::from_min_size(pos2(inner.min.x + (inner.width() - w) / 2.0, inner.min.y + (inner.height() - h) / 2.0), vec2(w, h));
            self.draw_sharp(p, "gear", "gear", r, Some(if on { th.accent() } else { th.text() }));
        }
        self.add_hit(settings, "settings", Act::Settings);

        // The introduction, between the title and the button, centred on the logo's middle.
        let intro_x = 800f32.max(title_x + self.width(p, "Font Converter", title) + 48.0);
        let runs = ["Pick a font for any text you want to change. ", "*Every size the game needs*", " is made from it."];
        let mut intro = Rect::from_min_size(pos2(intro_x, 0.0), vec2(settings.min.x - 28.0 - intro_x, 0.0));
        let intro_h = self.rich(p, intro, 16.0, th.muted(), th.text(), false, &runs);
        let y = (middle - intro_h / 2.0 - 2.0).max(8.0);
        intro = intro.translate(vec2(0.0, y));
        self.rich(p, intro, 16.0, th.muted(), th.text(), true, &runs);
    }

    /// Wrapped text with *bold* runs; its height. Measured only when `draw` is false.
    #[allow(clippy::too_many_arguments)]
    fn rich(&self, p: &egui::Painter, r: Rect, px: f32, colour: Color32, strong: Color32, draw: bool, runs: &[&str]) -> f32 {
        let (mut x, mut y) = (r.min.x, r.min.y);
        let line = self.theme.t(px) * 1.55;
        let font = body(px);
        for run in runs {
            let bold = run.starts_with('*');
            let text = run.trim_matches('*');
            for word in text.split_inclusive(' ') {
                let w = self.width(p, word, font);
                if x + w > r.max.x + 0.5 && x > r.min.x {
                    x = r.min.x;
                    y += line;
                }
                if draw {
                    self.draw_at(p, word, font, if bold { strong } else { colour }, x, y);
                }
                x += self.width(p, word.trim_end_matches(' '), font);
                if word.ends_with(' ') {
                    x += self.width(p, "a a", font) - self.width(p, "aa", font);
                }
            }
        }
        y - r.min.y + line
    }

    fn group_info(&self, name: &str) -> (Color32, &'static str) {
        let light = self.theme.light;
        match name {
            "Menus" => (self.theme.gold(), "Every screen you click through"),
            "Reading" => (if light { Color32::from_rgb(21, 101, 192) } else { Color32::from_rgb(96, 178, 255) }, "Conversations and the journal"),
            "HUD" => (self.theme.done(), "On screen while you play"),
            _ => (if light { Color32::from_rgb(118, 62, 184) } else { Color32::from_rgb(197, 140, 255) }, "The end credits"),
        }
    }

    fn paint_groups(&mut self, p: &egui::Painter) {
        let th = self.theme;
        let rows: [&[&str]; 2] = [&["Menus"], &["Reading", "HUD", "Credits"]];
        let mut y = 146.0;
        for row in rows {
            let count: usize = row.iter().map(|n| self.tiles.iter().filter(|t| t.def.group == *n).count()).sum();
            let available = 1480.0 - 14.0 * (row.len() as f32 - 1.0);
            let mut x = 60.0;
            for name in row {
                let members: Vec<usize> = (0..self.tiles.len()).filter(|&i| self.tiles[i].def.group == *name).collect();
                let w = available * members.len() as f32 / count as f32;
                let r = Rect::from_min_size(pos2(x, y), vec2(w, 291.0));
                self.rbox(p, r, 12.0, th.group(), th.group_edge(), 1.0);
                let (colour, line) = self.group_info(name);
                p.circle_filled(self.pt(x + 20.0, y + 22.0), 4.0 * self.scale, colour);
                let mut tx = x + 34.0;
                let head = display(15.0);
                for c in name.to_uppercase().chars() {
                    let s = c.to_string();
                    self.draw_at(p, &s, head, colour, tx, y + 13.0);
                    tx += self.width(p, &s, head) + 3.0;
                }
                let lr = Rect::from_min_size(pos2(tx + 9.0, y + 22.0 - th.t(11.0)), vec2(r.max.x - tx - 20.0, th.t(22.0)));
                self.draw(p, line, body(14.0), th.soft(), lr, false, false);
                let n = members.len() as f32;
                let tw = (w - 24.0 - 12.0 * (n - 1.0)) / n;
                for (i, &t) in members.iter().enumerate() {
                    let tr = Rect::from_min_size(pos2(x + 12.0 + i as f32 * (tw + 12.0), y + 43.0), vec2(tw, 236.0));
                    self.paint_tile(p, t, tr);
                }
                x += w + 14.0;
            }
            y += 291.0 + 14.0;
        }
    }

    fn paint_tile(&mut self, p: &egui::Painter, t: usize, r: Rect) {
        let th = self.theme;
        let mine = self.tiles[t].state.face.is_some();
        let id = format!("tile:{}", self.tiles[t].def.key);
        let lit = self.drop_target == Some(t);
        // A picked font lights its tile orange; once converted, green.
        let state = if self.tiles[t].state.converted { th.done() } else { th.picked() };
        let fill = if lit { th.tile_hover() } else { th.tile() };
        let edge = if lit { th.accent() } else if mine { state } else { th.tile_edge() };
        self.rbox(p, r, 8.0, fill, edge, if lit || mine { 2.0 } else { 1.0 });
        if mine && !lit {
            let pts = clip_below(&self.rounded_points(r, 8.0), self.py(r.min.y + r.height() * 0.6));
            fill_gradient(p, &pts, self.py(r.min.y), self.py(r.min.y + r.height() * 0.6), alpha(state, 48), alpha(state, 0));
        }

        let (name, use_) = (self.tiles[t].def.name, self.tiles[t].def.use_);
        self.draw(p, name, display(19.0), th.text(), Rect::from_min_size(pos2(r.min.x + 14.0, r.min.y + 10.0), vec2(r.width() - 28.0, th.t(26.0))), false, false);
        // What the tile is for, on two lines where a narrow tile needs them.
        let use_font = body(14.0);
        let (use_y, use_line) = (r.min.y + 10.0 + th.t(26.0), th.t(19.0));
        let use_lines = if self.width(p, use_, use_font) <= r.width() - 28.0 { 1.0 } else { 2.0 };
        self.draw_wrapped(p, use_, use_font, th.muted(), Rect::from_min_size(pos2(r.min.x + 14.0, use_y), vec2(r.width() - 28.0, use_line * use_lines + 2.0)));

        let foot_h = 34f32.max(th.t(30.0));
        let box_y = use_y + use_line * use_lines + 8.0;
        let bx = Rect::from_min_max(pos2(r.min.x + 14.0, box_y), pos2(r.max.x - 14.0, r.max.y - 12.0 - foot_h - 10.0));
        self.rbox(p, bx, 5.0, theme::EXAMPLE, th.example_edge(), 1.0);
        self.paint_example(p, t, bx);

        let foot = Rect::from_min_size(pos2(r.min.x + 14.0, r.max.y - 12.0 - foot_h), vec2(r.width() - 28.0, foot_h));
        if !mine {
            let choose = format!("{id}:choose");
            let on = self.is_hover(&choose);
            let pts = self.rounded_points(foot, 5.0);
            if on {
                p.add(Shape::convex_polygon(pts.clone(), th.button_hover(), Stroke::NONE));
            }
            let mut closed = pts;
            closed.push(closed[0]);
            let dash = Stroke::new(self.scale, if on { th.accent() } else { th.dashed() });
            p.extend(Shape::dashed_line(&closed, dash, 3.0 * self.scale, 2.0 * self.scale));
            self.draw(p, "Choose a font\u{2026}", display(15.0), if on { th.text() } else { th.choose_ink() }, foot, true, false);
            self.add_hit(foot, choose, Act::Choose(t));
        } else {
            let clear = Rect::from_min_size(pos2(foot.max.x - th.t(32.0), foot.min.y), vec2(th.t(32.0), foot.height()));
            let chip_id = format!("{id}:chip");
            let on = self.is_hover(&chip_id);
            self.rbox(p, foot, 5.0, alpha(state, if on { 40 } else { 22 }), alpha(state, 150), 1.0);
            // The font's name; a warning beside it only when the font lacks letters the game's
            // text uses (those are left blank).
            // The font's name, and where the font lacks letters the game's text uses (those are
            // left blank) a badge: the warning sign and how many, at the chip's right beside the
            // cross. Pointing at it shows which (2026-11-09, the maintainer: the warning "isn't
            // really shown fully"; until then "N missing" followed the name and cut it to a
            // letter or two in a narrow tile).
            let (name_font, small) = (display(15.0), display(14.0));
            let missing = self.tiles[t].state.missing.len();
            let sign = th.t(13.0);
            let count = missing.to_string();
            let badge_w = if missing == 0 { 0.0 } else { 8.0 + sign + 4.0 + self.width(p, &count, small) + 8.0 };
            let badge = Rect::from_min_size(pos2(clear.min.x - badge_w, foot.min.y + 5.0), vec2(badge_w, foot.height() - 10.0));
            let face_name = self.tiles[t].state.face.as_ref().map(|f| f.name.clone()).unwrap_or_default();
            let name_w = (foot.width() - 10.0 - clear.width() - badge_w - 6.0).max(20.0);
            self.draw(p, &face_name, name_font, state, Rect::from_min_size(pos2(foot.min.x + 10.0, foot.min.y), vec2(name_w, foot.height())), false, false);
            let badge_id = format!("{id}:missing");
            if missing > 0 {
                let on = self.is_hover(&badge_id);
                self.rbox(p, badge, 4.0, alpha(th.warning(), if on { 70 } else { 34 }), alpha(th.warning(), if on { 230 } else { 140 }), 1.0);
                let (sx, cy) = (badge.min.x + 8.0, badge.center().y);
                // The warning sign drawn as a triangle: no font the window carries has one.
                let tri = vec![self.pt(sx + sign / 2.0, cy - sign * 0.45), self.pt(sx + sign, cy + sign * 0.42), self.pt(sx, cy + sign * 0.42)];
                p.add(Shape::convex_polygon(tri, th.warning(), Stroke::NONE));
                let bang = Stroke::new(sign * 0.12 * self.scale, theme::EXAMPLE);
                p.line_segment([self.pt(sx + sign / 2.0, cy - sign * 0.12), self.pt(sx + sign / 2.0, cy + sign * 0.16)], bang);
                p.circle_filled(self.pt(sx + sign / 2.0, cy + sign * 0.29), sign * 0.06 * self.scale, theme::EXAMPLE);
                let wr = Rect::from_min_max(pos2(sx + sign + 4.0, badge.min.y), badge.max);
                self.draw(p, &count, small, th.warning(), wr, false, false);
                if on {
                    self.tooltip = Some((badge, t));
                }
            }
            let clear_id = format!("{id}:clear");
            let x_on = self.is_hover(&clear_id);
            // The cross drawn as two strokes: KMRP's fonts have only the game's letters, and the
            // fonts egui carries have no cross either.
            let (c, arm) = (clear.center(), th.t(5.0));
            let cross = Stroke::new(1.4 * self.scale, if x_on { th.text() } else { th.faint() });
            p.line_segment([self.pt(c.x - arm, c.y - arm), self.pt(c.x + arm, c.y + arm)], cross);
            p.line_segment([self.pt(c.x - arm, c.y + arm), self.pt(c.x + arm, c.y - arm)], cross);
            self.add_hit(foot, chip_id, Act::Choose(t));
            self.add_hit(clear, clear_id, Act::Clear(t));
            if missing > 0 {
                self.add_hit(badge, badge_id, Act::Choose(t));
            }
        }
    }

    /// The pop-up over a tile's missing-letters badge: how many, and which.
    fn paint_tooltip(&mut self, p: &egui::Painter, anchor: Rect, t: usize) {
        let th = self.theme;
        let missing = &self.tiles[t].state.missing;
        let n = missing.len();
        let title = format!("{n} letter{} missing", if n == 1 { "" } else { "s" });
        // The letters themselves, in the window's own text face, which has all of the game's
        // letters; the invisible ones by name.
        let letters: Vec<String> = missing
            .iter()
            .map(|c| match *c {
                ' ' => "space".to_string(),
                '\u{A0}' => "no-break space".to_string(),
                '\u{AD}' => "soft hyphen".to_string(),
                c => c.to_string(),
            })
            .collect();
        let text = format!(
            "This font has no {}. The game's text leaves {} blank; every other letter is drawn in this font.",
            letters.join("  "),
            if n == 1 { "it" } else { "them" }
        );
        let width = 340.0;
        let font = body(14.0);
        let g = self.wrapped_galley(p, &text, font, th.muted(), width - 28.0);
        let lines = g.rows.len().max(1) as f32;
        let line = th.t(14.0) * 1.4;
        let h = 12.0 + th.t(22.0) + 4.0 + lines * line + 12.0;
        // Above the badge where there is room, else below it; inside the canvas.
        let x = (anchor.center().x - width / 2.0).clamp(8.0, CW - width - 8.0);
        let y = if anchor.min.y - h - 8.0 > 8.0 { anchor.min.y - h - 8.0 } else { anchor.max.y + 8.0 };
        let r = Rect::from_min_size(pos2(x, y), vec2(width, h));
        let shadow = Rect::from_min_size(pos2(r.min.x - 2.0, r.min.y + 6.0), vec2(r.width() + 4.0, r.height() + 4.0));
        p.rect_filled(self.rect(shadow), 10.0 * self.scale, th.shadow());
        self.rbox(p, r, 8.0, th.panel(), th.warning(), 1.0);
        self.draw(p, &title, display(16.0), th.warning(), Rect::from_min_size(pos2(r.min.x + 14.0, r.min.y + 12.0), vec2(width - 28.0, th.t(22.0))), false, false);
        let top = r.min.y + 12.0 + th.t(22.0) + 4.0;
        for (i, row) in g.rows.iter().enumerate() {
            let words: String = row.row.glyphs.iter().map(|gl| gl.chr).collect();
            self.draw_at(p, words.trim_end(), font, th.muted(), r.min.x + 14.0, top + i as f32 * line);
        }
    }

    /// A piece's width at its own height, on the canvas.
    fn piece_width(&mut self, t: usize, l: usize, k: usize) -> f32 {
        let (name, height) = {
            let piece = &self.tiles[t].def.lines[l][k];
            (piece.name.clone(), piece.height)
        };
        if self.tiles[t].state.face.is_none() {
            return self.examples.get(&name).map(|i| i.w as f32 * height / i.h as f32).unwrap_or(height);
        }
        // Measured at twice its canvas height, once a size.
        let h = ((height * 2.0).round() as i32).max(4);
        let key = (t, name);
        if !self.measures.get(&key).is_some_and(|m| m.0 == h) {
            let piece = &self.tiles[t].def.lines[l][k];
            let face = self.tiles[t].state.face.clone().unwrap();
            let ring = ring_for(piece, h as f32);
            let measured = Some(atlas::line(&face, &piece.text, h as f32, piece.colour, ring)).map(|i| (i.width, i.pad));
            self.measures.insert(key.clone(), (h, measured));
        }
        match self.measures[&key].1 {
            Some((w, pad)) => (w as f32 - 2.0 * (pad as f32 - 1.0)) / 2.0,
            None => height,
        }
    }

    // The tile's example lines, each piece in the HD Original font or the picked one, all of a
    // tile's lines shrunk together when the widest would not fit.
    fn paint_example(&mut self, p: &egui::Painter, t: usize, bx: Rect) {
        let mine = self.tiles[t].state.face.is_some();
        let (gap, line_gap) = (18.0, 9.0);
        let room = bx.width() - 24.0;
        let lines = self.tiles[t].def.lines.len();
        let mut widths = Vec::new();
        for l in 0..lines {
            let n = self.tiles[t].def.lines[l].len();
            let mut w = gap * (n as f32 - 1.0);
            for k in 0..n {
                w += self.piece_width(t, l, k);
            }
            widths.push(w);
        }
        let line_h = |line: &Vec<Piece>| line.iter().map(|p| p.height).fold(0.0, f32::max);
        let tall: f32 = self.tiles[t].def.lines.iter().map(line_h).sum();
        let gaps = line_gap * (lines as f32 - 1.0);
        let widest = widths.iter().copied().fold(1.0, f32::max);
        let shrink = 1f32.min((room / widest).min((bx.height() - 16.0 - gaps) / tall.max(1.0)));
        let total = tall * shrink + gaps;
        let mut y = bx.min.y + (bx.height() - total) / 2.0;
        let centre = self.tiles[t].def.centre;
        for l in 0..lines {
            let lh = line_h(&self.tiles[t].def.lines[l]) * shrink;
            let mut x = if centre { bx.min.x + (bx.width() - widths[l] * shrink) / 2.0 } else { bx.min.x + 12.0 };
            for k in 0..self.tiles[t].def.lines[l].len() {
                let (h, name) = {
                    let piece = &self.tiles[t].def.lines[l][k];
                    (piece.height * shrink, piece.name.clone())
                };
                let w = self.piece_width(t, l, k) * shrink;
                let r = Rect::from_min_size(pos2(x, y + (lh - h) / 2.0), vec2(w, h));
                if mine {
                    self.draw_preview(p, t, l, k, r);
                } else {
                    self.draw_sharp(p, &name, &name, r, None);
                }
                x += w + gap * shrink;
            }
            y += lh + line_gap;
        }
    }

    // A piece in the picked font, as high as `r` on the screen. One texture a piece, made
    // again when the size changes.
    fn draw_preview(&mut self, p: &egui::Painter, t: usize, l: usize, k: usize, r: Rect) {
        let h = ((r.height() * self.device()).round() as i32).max(4);
        let name = self.tiles[t].def.lines[l][k].name.clone();
        let key = (t, name.clone());
        if !self.previews.get(&key).is_some_and(|x| x.box_h == h) {
            let piece = &self.tiles[t].def.lines[l][k];
            let Some(face) = self.tiles[t].state.face.clone() else { return };
            let img = atlas::line(&face, &piece.text, h as f32, piece.colour, ring_for(piece, h as f32));
            let handle = self.texture(&format!("preview:{t}:{name}"), img.width, img.height, &img.rgba, true);
            self.previews.insert(key.clone(), Tex { handle, w: img.width, h: img.height, box_h: img.box_height, pad: img.pad });
        }
        let tex = &self.previews[&key];
        let m = (tex.pad as f32 - 1.0) / self.ppp;
        self.draw_texture(p, tex, self.px(r.min.x) - m, self.py(r.min.y) - m);
    }

    fn paint_actions(&mut self, p: &egui::Painter) {
        let th = self.theme;
        let chosen = self.tiles.iter().filter(|t| t.state.face.is_some()).count();
        let busy = self.busy();
        let count = display(20.0);
        // The bottom row: 86 high at y 760 since 2026-11-09.
        let (row, row_h) = (760.0, 86.0);
        let middle = row + row_h / 2.0;
        let mut x = 60.0;
        let n = chosen.to_string();
        self.draw(p, &n, count, th.gold(), Rect::from_min_size(pos2(x, middle - 20.0), vec2(60.0, 40.0)), false, false);
        // The font's own word space after the number.
        x += self.width(p, &n, count) + self.width(p, "a a", count) - self.width(p, "aa", count);
        let of_all = "of 11 fonts chosen";
        let of_w = self.width(p, of_all, count);
        self.draw(p, of_all, count, th.text(), Rect::from_min_size(pos2(x, middle - 20.0), vec2(of_w + 4.0, 40.0)), false, false);
        x += of_w + 24.0;

        // Convert, the Version box and Clear from the right, each as wide as its longest label
        // needs; the folder takes what is left.
        let (big, mid, choice) = (display(21.0), display(19.0), display(14.0));
        let convert_w = 220f32.max(self.width(p, "Convert", big).max(th.t(17.0) + 10.0 + self.width(p, "Converted", big)) + 64.0);
        let version_w = 150f32.max(44.0 + self.width(p, "Original Game", choice).max(self.width(p, "KMRP", choice)) + 16.0);
        let clear_w = 120f32.max(self.width(p, "Clear", mid) + 50.0);
        let convert = Rect::from_min_size(pos2(1540.0 - convert_w, row), vec2(convert_w, row_h));
        let version = Rect::from_min_size(pos2(convert.min.x - 10.0 - version_w, row), vec2(version_w, row_h));
        let clear = Rect::from_min_size(pos2(version.min.x - 14.0 - clear_w, row), vec2(clear_w, row_h));

        let path = Rect::from_min_max(pos2(x, row), pos2(clear.min.x - 14.0, row + row_h));
        self.rbox(p, path, 5.0, th.field(), th.edge(), 1.0);
        let change = display(16.0);
        let change_w = 86f32.max(self.width(p, "Change", change) + 28.0);
        let change_box = Rect::from_min_size(pos2(path.max.x - 14.0 - change_w, path.min.y + (row_h - 38.0) / 2.0), vec2(change_w, 38.0));
        let (label_h, path_h) = (th.t(16.0), th.t(22.0));
        let top = path.min.y + (path.height() - label_h - path_h) / 2.0;
        self.draw(p, "SAVE TO", body(12.5), th.faint(), Rect::from_min_size(pos2(path.min.x + 16.0, top), vec2(200.0, label_h)), false, false);
        let shown = self.output.to_string_lossy().into_owned();
        let pr = Rect::from_min_size(pos2(path.min.x + 16.0, top + label_h), vec2(change_box.min.x - path.min.x - 30.0, path_h));
        self.draw(p, &shown, mono(14.0), th.muted(), pr, false, true);
        let on = !busy && self.is_hover("output");
        self.rbox(p, change_box, 5.0, if on { th.button_hover() } else { th.button() }, if on { th.accent() } else { th.edge() }, 1.0);
        self.draw(p, "Change", change, if busy { th.faint() } else { th.text() }, change_box, true, false);
        if !busy {
            self.add_hit(change_box, "output", Act::PickFolder);
        }

        // Clear: every tile back to the Original HD Font. A grey button: set apart from the
        // folder's field, quieter than Convert.
        let can_clear = chosen > 0 && !busy;
        let on_clear = can_clear && self.is_hover("clearall");
        let rgb = Color32::from_rgb;
        let (clear_top, clear_bottom, clear_edge, off_ink) = if th.light {
            (
                if can_clear { if on_clear { rgb(214, 222, 231) } else { rgb(201, 210, 220) } } else { rgb(226, 232, 239) },
                if can_clear { if on_clear { rgb(195, 204, 214) } else { rgb(182, 192, 203) } } else { rgb(218, 225, 233) },
                rgb(150, 165, 182),
                rgb(150, 162, 176),
            )
        } else {
            (
                if can_clear { if on_clear { rgb(104, 117, 134) } else { rgb(86, 98, 114) } } else { rgb(40, 48, 60) },
                if can_clear { if on_clear { rgb(82, 94, 110) } else { rgb(66, 77, 92) } } else { rgb(32, 39, 50) },
                rgb(108, 122, 140),
                rgb(92, 104, 118),
            )
        };
        self.gradient(p, clear, 5.0, clear_top, clear_bottom, None);
        self.rbox(p, clear, 5.0, Color32::TRANSPARENT, clear_edge, 1.0);
        self.draw(p, "Clear", mid, if can_clear { th.text() } else { off_ink }, clear, true, false);
        if can_clear {
            self.add_hit(clear, "clearall", Act::ClearAll);
        }

        self.paint_version(p, version, busy);

        // Convert, or Converted while every picked font is written to this folder as it is.
        let done = chosen > 0 && !busy && self.converted;
        let ready = chosen > 0 && !busy && !done;
        if ready {
            let hi = if self.is_hover("convert") { ACCENT_HOVER } else { ACCENT_TOP };
            self.gradient(p, convert, 5.0, hi, ACCENT_BOTTOM, None);
        } else if done {
            self.gradient(p, convert, 5.0, rgb(146, 234, 188), rgb(96, 204, 148), None);
        } else {
            let (fill, edge) = if th.light { (rgb(212, 228, 238), rgb(150, 180, 200)) } else { (rgb(18, 40, 54), rgb(44, 84, 106)) };
            self.rbox(p, convert, 5.0, fill, edge, 1.0);
        }
        let (progress, total) = match &self.job {
            Some(j) => (j.progress.load(Ordering::SeqCst), j.total),
            None => (0, 0),
        };
        if busy && total > 0 {
            let cut = convert.min.x + convert.width() * progress as f32 / total as f32;
            self.gradient(p, convert, 5.0, ACCENT_TOP, ACCENT_BOTTOM, Some(cut));
        }
        let waiting = if th.light { rgb(110, 140, 160) } else { rgb(70, 110, 130) };
        let ink = if ready || busy || done { ACCENT_INK } else { waiting };
        if done {
            // The tick drawn as two strokes: KMRP's fonts have only the game's letters.
            let text_w = self.width(p, "Converted", big);
            let tick = th.t(17.0);
            let tx = convert.min.x + (convert.width() - tick - 10.0 - text_w) / 2.0;
            let cy = convert.center().y;
            let pts = vec![self.pt(tx, cy), self.pt(tx + tick * 0.36, cy + tick * 0.4), self.pt(tx + tick, cy - tick * 0.48)];
            let width = 2.5f32.max(th.t(2.6)) * self.scale;
            for q in &pts {
                p.circle_filled(*q, width / 2.0, ink);
            }
            p.add(Shape::line(pts, Stroke::new(width, ink)));
            self.draw(p, "Converted", big, ink, Rect::from_min_size(pos2(tx + tick + 10.0, convert.min.y), vec2(text_w + 6.0, convert.height())), false, false);
        } else {
            // The progress shrunk to fit Convert, which is as wide as "Converted" needs.
            let label = if busy { format!("Converting\u{2026} {progress} of {total}") } else { "Convert".to_string() };
            let label_w = self.width(p, &label, big);
            let room = convert.width() - 24.0;
            let font = if label_w > room { display(21.0 * room / label_w) } else { big };
            self.draw(p, &label, font, ink, convert, true, false);
        }
        if ready {
            self.add_hit(convert, "convert", Act::Convert);
        }

        // The footer line, each piece after the last.
        let foot = body(15.0);
        let (fy, fh) = (876.0 - th.t(12.0), th.t(24.0));
        let mut fx = 62.0;
        let version_text = format!("v{}", crate::VERSION);
        let vw = self.width(p, &version_text, foot);
        self.draw(p, &version_text, foot, th.faint(), Rect::from_min_size(pos2(fx, fy), vec2(vw + 4.0, fh)), false, false);
        fx += vw + 12.0;
        self.draw(p, "\u{00B7}", foot, th.faint(), Rect::from_min_size(pos2(fx, fy), vec2(12.0, fh)), false, false);
        fx += self.width(p, "\u{00B7}", foot) + 12.0;
        let open_w = self.width(p, "Open Output Folder", foot);
        let open = Rect::from_min_size(pos2(fx, fy), vec2(open_w + 4.0, fh));
        self.draw(p, "Open Output Folder", foot, if self.is_hover("open") { th.text() } else { th.accent() }, open, false, false);
        self.add_hit(open, "open", Act::OpenOutput);
        let credit = "Created by RaymanGT";
        let cw = self.width(p, credit, foot);
        self.draw(p, credit, foot, th.faint(), Rect::from_min_size(pos2(1538.0 - cw, fy), vec2(cw + 4.0, fh)), false, false);
    }

    // The Version box: "Version" in the window's yellow, then KMRP and Original Game with
    // KOTOR's round radio rings in its own blue.
    fn paint_version(&mut self, p: &egui::Painter, bx: Rect, busy: bool) {
        let th = self.theme;
        self.rbox(p, bx, 5.0, th.field(), th.edge(), 1.0);
        let (label, choice) = (body(15.0), display(14.0));
        let label_h = th.t(19.0);
        let rows_top = bx.min.y + 7.0 + label_h;
        let row_h = (bx.max.y - 7.0 - rows_top) / 2.0;
        self.draw(p, "Version", label, th.gold(), Rect::from_min_size(pos2(bx.min.x + 12.0, bx.min.y + 6.0), vec2(bx.width() - 20.0, label_h)), false, false);
        for (i, (t, text)) in [(Target::Kmrp, "KMRP"), (Target::Original, "Original Game")].into_iter().enumerate() {
            let on = self.target == t;
            let id = if t == Target::Kmrp { "version:kmrp" } else { "version:original" };
            let lit = !busy && !on && self.is_hover(id);
            let row = Rect::from_min_size(pos2(bx.min.x + 4.0, rows_top + i as f32 * row_h), vec2(bx.width() - 8.0, row_h));
            // 3.5 each side of a ring, less half the pen: 5 pixels clear between the two rings.
            let ring = th.t(8.0).min(row_h / 2.0 - 3.5);
            let (cx, cy) = (bx.min.x + 12.0 + ring, row.min.y + row_h / 2.0);
            let pen = 1.6f32.max(ring / 3.6);
            p.circle_stroke(self.pt(cx, cy), ring * self.scale, Stroke::new(pen * self.scale, if lit { th.text() } else { th.kotor_blue() }));
            if on {
                p.circle_filled(self.pt(cx, cy), ring * 0.5 * self.scale, th.kotor_blue());
            }
            let tr = Rect::from_min_max(pos2(cx + ring + 9.0, row.min.y), pos2(row.max.x, row.max.y));
            self.draw(p, text, choice, if on || lit { th.text() } else { th.faint() }, tr, false, false);
            if !busy && !on {
                self.add_hit(row, id, Act::SetTarget(t));
            }
        }
    }

    // ---- the pop-ups ----

    fn frame(&self, p: &egui::Painter, r: Rect) {
        let shadow = Rect::from_min_size(pos2(r.min.x - 4.0, r.min.y + 10.0), vec2(r.width() + 8.0, r.height() + 8.0));
        p.rect_filled(self.rect(shadow), 16.0 * self.scale, self.theme.shadow());
        self.rbox(p, r, 12.0, self.theme.panel(), self.theme.edge(), 1.0);
    }

    fn paint_buttons(&mut self, p: &egui::Painter, buttons: &[(String, bool, DAct)], r: Rect, y: f32) {
        let th = self.theme;
        let font = display(18.0);
        let width = |s: &Self, label: &str, primary: bool| (if primary { 200f32 } else { 130.0 }).max(s.width(p, label, font) + 60.0);
        // One button sits in the middle; more line up on the right.
        let mut x = r.max.x - 28.0;
        if buttons.len() == 1 {
            x = r.min.x + r.width() / 2.0 + width(self, &buttons[0].0, buttons[0].1) / 2.0;
        }
        for i in (0..buttons.len()).rev() {
            let (label, primary, _) = &buttons[i];
            let w = width(self, label, *primary);
            let b = Rect::from_min_size(pos2(x - w, y), vec2(w, 48.0));
            let id = format!("button{i}");
            let on = self.is_hover(&id);
            if *primary {
                self.gradient(p, b, 5.0, if on { ACCENT_HOVER } else { ACCENT_TOP }, ACCENT_BOTTOM, None);
            } else {
                self.rbox(p, b, 5.0, if on { th.button_hover() } else { th.button() }, if on { th.accent() } else { th.edge() }, 1.0);
            }
            self.draw(p, label, font, if *primary { ACCENT_INK } else { th.text() }, b, true, false);
            self.add_hit(b, id, Act::Button(i));
            x -= w + 12.0;
        }
    }

    fn paint_dialog(&mut self, p: &egui::Painter) {
        let Some(dialog) = self.dialog.take() else { return };
        let mut dialog = dialog;
        match &mut dialog.kind {
            Kind::Message => self.paint_message(p, &dialog),
            Kind::Settings => self.paint_settings(p, &dialog),
            Kind::Picker(picker) => {
                let (title, width, buttons) = (dialog.title.clone(), dialog.width, dialog.buttons.clone());
                self.paint_picker(p, picker, &title, width, &buttons);
            }
        }
        self.dialog = Some(dialog);
    }

    fn paint_message(&mut self, p: &egui::Painter, d: &Dialog) {
        let th = self.theme;
        let font = body(16.0);
        let lines = if d.body.is_empty() { 0 } else { self.wrapped_galley(p, &d.body, font, th.muted(), d.width - 56.0).rows.len().max(1) };
        let body_h = lines as f32 * th.t(16.0) * 1.5;
        let h = 26.0 + th.t(34.0) + 10.0 + body_h + 24.0 + 48.0 + 24.0;
        let r = Rect::from_min_size(pos2((CW - d.width) / 2.0, (CH - h) / 2.0), vec2(d.width, h));
        self.frame(p, r);
        let mut y = r.min.y + 26.0;
        self.draw(p, &d.title, display(26.0), th.text(), Rect::from_min_size(pos2(r.min.x + 28.0, y), vec2(r.width() - 56.0, th.t(34.0))), false, false);
        y += th.t(34.0) + 10.0;
        // Lines spaced as the C# program spaced them: 1.5 times the size.
        let g = self.wrapped_galley(p, &d.body, font, th.muted(), r.width() - 56.0);
        let step = th.t(16.0) * 1.5 * self.scale;
        for (i, row) in g.rows.iter().enumerate() {
            let text: String = row.row.glyphs.iter().map(|gl| gl.chr).collect();
            let line = self.galley(p, text.trim_end(), font, th.muted());
            let y0 = self.py(y) + i as f32 * step + (step - line.size().y) / 2.0;
            p.galley(pos2(self.snap(self.px(r.min.x + 28.0)), self.snap(y0)), line, th.muted());
        }
        y += body_h + 24.0;
        let buttons = d.buttons.clone();
        self.paint_buttons(p, &buttons, r, y);
    }

    fn settings_dialog() -> Dialog {
        Dialog { kind: Kind::Settings, title: "Settings".into(), body: String::new(), width: 600.0, buttons: vec![("Close".into(), true, DAct::Close)] }
    }

    fn paint_settings(&mut self, p: &egui::Painter, d: &Dialog) {
        let th = self.theme;
        let seg_h = 48f32.max(th.t(38.0));
        let label_h = th.t(18.0);
        let h = 26.0 + th.t(34.0) + 14.0 + (label_h + 10.0 + seg_h) * 2.0 + 24.0 + 28.0 + 48.0 + 24.0;
        let r = Rect::from_min_size(pos2((CW - d.width) / 2.0, (CH - h) / 2.0), vec2(d.width, h));
        self.frame(p, r);
        let mut y = r.min.y + 26.0;
        self.draw(p, &d.title, display(26.0), th.text(), Rect::from_min_size(pos2(r.min.x + 28.0, y), vec2(r.width() - 56.0, th.t(34.0))), false, false);
        y += th.t(34.0) + 14.0;
        let size = places::TEXT_SCALES.iter().position(|s| (*s - th.text_scale).abs() < 0.001).unwrap_or(1);
        y = self.section(p, r, y, "TEXT SIZE", &["Small", "Medium", "Large"], size, label_h, seg_h, false) + 24.0;
        self.section(p, r, y, "APPEARANCE", &["Dark", "Light"], usize::from(th.light), label_h, seg_h, true);
        let buttons = d.buttons.clone();
        self.paint_buttons(p, &buttons, r, r.max.y - 24.0 - 48.0);
    }

    // A spaced label, then a row of choices with the picked one lit.
    #[allow(clippy::too_many_arguments)]
    fn section(&mut self, p: &egui::Painter, r: Rect, y: f32, label: &str, choices: &[&str], picked: usize, label_h: f32, seg_h: f32, icons: bool) -> f32 {
        let th = self.theme;
        let head = display(14.0);
        let mut tx = r.min.x + 28.0;
        for c in label.chars() {
            let s = c.to_string();
            self.draw_at(p, &s, head, th.accent(), tx, y);
            tx += self.width(p, &s, head) + if c == ' ' { 6.0 } else { 3.0 };
        }
        let y = y + label_h + 10.0;
        let n = choices.len() as f32;
        let w = (r.width() - 56.0 - 10.0 * (n - 1.0)) / n;
        let font = display(18.0);
        for (i, choice) in choices.iter().enumerate() {
            let b = Rect::from_min_size(pos2(r.min.x + 28.0 + i as f32 * (w + 10.0), y), vec2(w, seg_h));
            let id = format!("{label}{i}");
            let (on, lit) = (i == picked, self.is_hover(&id));
            if on {
                self.gradient(p, b, 5.0, ACCENT_TOP, ACCENT_BOTTOM, None);
            } else {
                self.rbox(p, b, 5.0, if lit { th.button_hover() } else { th.button() }, if lit { th.accent() } else { th.edge() }, 1.0);
            }
            let ink = if on { ACCENT_INK } else { th.text() };
            if !icons {
                self.draw(p, choice, font, ink, b, true, false);
            } else {
                // The moon or the sun before the word, the pair centred.
                let icon_s = 20f32.max(th.t(20.0));
                let text_w = self.width(p, choice, font);
                let x = b.min.x + (b.width() - icon_s - 10.0 - text_w) / 2.0;
                let ib = Rect::from_min_size(pos2(x, b.min.y + (b.height() - icon_s) / 2.0), vec2(icon_s, icon_s));
                let colour = if on { ACCENT_INK } else { th.gold() };
                if i == 0 {
                    self.moon(p, ib, colour);
                } else {
                    self.sun(p, ib, colour);
                }
                self.draw(p, choice, font, ink, Rect::from_min_size(pos2(x + icon_s + 10.0, b.min.y), vec2(text_w + 6.0, b.height())), false, false);
            }
            let act = if icons { Act::Light(i == 1) } else { Act::TextSize(i) };
            self.add_hit(b, id, act);
        }
        y + seg_h
    }

    // The Save Editor's crescent: a disc less a disc offset up and right, filled at the
    // screen's pixels.
    fn moon(&mut self, p: &egui::Painter, b: Rect, colour: Color32) {
        let s = self.device();
        let (w, h) = ((b.width() * s).round().max(1.0) as usize, (b.height() * s).round().max(1.0) as usize);
        let key = format!("moon:{w}x{h}:{:?}", colour.to_array());
        if !self.scaled.get(&key).is_some_and(|t| t.w == w) {
            let d = b.width() * 0.86 * s;
            let (cx, cy, r) = (w as f32 / 2.0, h as f32 / 2.0, d / 2.0);
            let c = d * 0.78;
            let (kx, ky, kr) = ((b.width() * s - d) / 2.0 + d * 0.38 + c / 2.0, (b.height() * s - d) / 2.0 - d * 0.16 + c / 2.0, c / 2.0);
            // Each pixel the share of its 16 samples inside the disc and outside the cut.
            let mut cover = vec![0u8; w * h];
            for y in 0..h {
                for x in 0..w {
                    let mut n = 0;
                    for sy in 0..4 {
                        for sx in 0..4 {
                            let (px, py) = (x as f32 + (sx as f32 + 0.5) / 4.0, y as f32 + (sy as f32 + 0.5) / 4.0);
                            let disc = (px - cx).powi(2) + (py - cy).powi(2) < r * r;
                            let cut = (px - kx).powi(2) + (py - ky).powi(2) < kr * kr;
                            if disc && !cut {
                                n += 1;
                            }
                        }
                    }
                    cover[y * w + x] = (n * 255 / 16) as u8;
                }
            }
            let mut rgba = vec![0u8; w * h * 4];
            for (i, a) in cover.iter().enumerate() {
                let a = *a as u32;
                rgba[i * 4] = (colour.r() as u32 * a / 255) as u8;
                rgba[i * 4 + 1] = (colour.g() as u32 * a / 255) as u8;
                rgba[i * 4 + 2] = (colour.b() as u32 * a / 255) as u8;
                rgba[i * 4 + 3] = a as u8;
            }
            let handle = self.texture(&key, w, h, &rgba, true);
            self.scaled.insert(key.clone(), Tex { handle, w, h, box_h: 0, pad: 0 });
        }
        let tex = &self.scaled[&key];
        self.draw_texture(p, tex, self.px(b.min.x), self.py(b.min.y));
    }

    // The sun: a disc with eight short rays.
    fn sun(&self, p: &egui::Painter, b: Rect, colour: Color32) {
        let (cx, cy) = (b.center().x, b.center().y);
        let r = b.width() * 0.2;
        p.circle_filled(self.pt(cx, cy), r * self.scale, colour);
        let width = 1.5f32.max(b.width() / 12.0) * self.scale;
        for i in 0..8 {
            let a = i as f32 * std::f32::consts::PI / 4.0;
            let (inner, outer) = (b.width() * 0.32, b.width() * 0.47);
            let (p0, p1) = (self.pt(cx + a.cos() * inner, cy + a.sin() * inner), self.pt(cx + a.cos() * outer, cy + a.sin() * outer));
            p.line_segment([p0, p1], Stroke::new(width, colour));
            p.circle_filled(p0, width / 2.0, colour);
            p.circle_filled(p1, width / 2.0, colour);
        }
    }

    fn row_h(&self) -> f32 {
        12.0 + self.theme.t(20.0) + 34.0 + 10.0
    }

    fn paint_picker(&mut self, p: &egui::Painter, picker: &mut Picker, title: &str, width: f32, buttons: &[(String, bool, DAct)]) {
        let th = self.theme;
        let row_h = self.row_h();
        let rows = VISIBLE.min(picker.faces.len());
        let list_h = rows as f32 * row_h + (rows as f32 - 1.0) * 8.0;
        let h = 26.0 + th.t(34.0) + 8.0 + th.t(24.0) + 18.0 + list_h + 22.0 + 48.0 + 24.0;
        let r = Rect::from_min_size(pos2((CW - width) / 2.0, (CH - h) / 2.0), vec2(width, h));
        self.frame(p, r);
        let mut y = r.min.y + 26.0;
        self.draw(p, title, display(26.0), th.text(), Rect::from_min_size(pos2(r.min.x + 28.0, y), vec2(r.width() - 56.0, th.t(34.0))), false, false);
        y += th.t(34.0) + 8.0;
        let plain = body(15.5);
        let (a, c) = ("Pick the one to use for ", ". Each is shown in its own letters.");
        let mut x = r.min.x + 28.0;
        self.draw_at(p, a, plain, th.muted(), x, y);
        x += self.width(p, a.trim_end(), plain) + self.width(p, "a a", plain) - self.width(p, "aa", plain);
        let name = self.tiles[picker.tile].def.name;
        self.draw_at(p, name, plain, th.text(), x, y);
        x += self.width(p, name, plain);
        self.draw_at(p, c, plain, th.muted(), x, y);
        y += th.t(24.0) + 18.0;

        let list = Rect::from_min_size(pos2(r.min.x + 28.0, y), vec2(r.width() - 56.0 - if picker.faces.len() > VISIBLE { 14.0 } else { 0.0 }, list_h));
        for i in 0..rows {
            let n = picker.top + i;
            let row = Rect::from_min_size(pos2(list.min.x, y + i as f32 * (row_h + 8.0)), vec2(list.width(), row_h));
            let id = format!("row{n}");
            let (on, lit) = (n == picker.selected, self.is_hover(&id));
            let fill = if on { th.selected() } else if lit { th.button_hover() } else { th.window() };
            self.rbox(p, row, 7.0, fill, if on { th.accent() } else { th.tile_edge() }, 1.0);
            let dot = self.pt(row.min.x + 25.0, row.min.y + row_h / 2.0);
            p.circle_stroke(dot, 9.0 * self.scale, Stroke::new(2.0 * self.scale, if on { th.accent() } else { th.radio() }));
            if on {
                p.circle_filled(dot, 4.5 * self.scale, th.accent());
            }
            let nr = Rect::from_min_size(pos2(row.min.x + 50.0, row.min.y + 9.0), vec2(row.width() - 66.0, th.t(20.0)));
            self.draw(p, &picker.faces[n].name, display(16.0), if on { th.accent() } else { th.soft() }, nr, false, false);
            if self.sample_for(picker, n, 34.0 * self.device()) {
                let clip = self.rect(Rect::from_min_size(pos2(row.min.x + 50.0, row.min.y), vec2(row.width() - 62.0, row.height())));
                let clipped = p.with_clip_rect(clip.intersect(p.clip_rect()));
                if let Some(tex) = picker.samples[&n].1.as_ref() {
                    self.draw_texture(&clipped, tex, self.px(row.min.x + 49.0), self.py(row.min.y + 12.0 + th.t(20.0)));
                }
            }
            self.add_hit(row, id, Act::Row(n));
        }
        if picker.faces.len() > VISIBLE {
            let rail = Rect::from_min_size(pos2(list.max.x + 6.0, y), vec2(4.0, list_h));
            self.rbox(p, rail, 2.0, th.tile_edge(), Color32::TRANSPARENT, 1.0);
            let thumb_h = list_h * VISIBLE as f32 / picker.faces.len() as f32;
            let thumb_y = y + (list_h - thumb_h) * picker.top as f32 / (picker.faces.len() - VISIBLE) as f32;
            self.rbox(p, Rect::from_min_size(pos2(rail.min.x - 1.0, thumb_y), vec2(6.0, thumb_h)), 3.0, th.accent(), Color32::TRANSPARENT, 1.0);
        }
        self.paint_buttons(p, buttons, r, y + list_h + 22.0);
    }

    /// A face's sample line, made again at a new height or theme; whether there is one.
    fn sample_for(&self, picker: &mut Picker, n: usize, height: f32) -> bool {
        let key = format!("{}{}", height as i32, if self.theme.light { "l" } else { "d" });
        if picker.samples.get(&n).is_some_and(|s| s.0 == key) {
            return picker.samples[&n].1.is_some();
        }
        let face = picker.loaded.entry(n).or_insert_with(|| font::load(&picker.faces[n]).ok()).clone();
        let tex = face.map(|face| {
            let colour = if self.theme.light { [13, 26, 43] } else { [200, 226, 250] };
            let img = atlas::line(&face, &picker.sample, height, colour, 0.0);
            let handle = self.texture(&format!("sample:{n}"), img.width, img.height, &img.rgba, true);
            Tex { handle, w: img.width, h: img.height, box_h: img.box_height, pad: img.pad }
        });
        let made = tex.is_some();
        picker.samples.insert(n, (key, tex));
        made
    }

    fn picker(&self, tile: usize, faces: Vec<FontFaceChoice>, reopen: bool) -> Dialog {
        let styles = faces.iter().any(|f| f.axes.is_some());
        let file = faces[0].path.file_name().map(|n| n.to_string_lossy().into_owned()).unwrap_or_default();
        let title = format!("{file} holds {} {}", faces.len(), if styles { "styles" } else { "fonts" });
        let sample = self.tiles[tile]
            .def
            .lines
            .iter()
            .map(|l| l.iter().map(|p| p.text.as_str()).collect::<Vec<_>>().join(" "))
            .collect::<Vec<_>>()
            .join(" ");
        let mut picker = Picker { tile, faces, selected: 0, top: 0, reopen, sample, loaded: HashMap::new(), samples: HashMap::new() };
        if reopen {
            if let Some(choice) = &self.tiles[tile].state.choice {
                if let Some(at) = picker.faces.iter().position(|f| f.face_index == choice.face_index && f.name == choice.name) {
                    picker.selected = at;
                    picker.top = at.saturating_sub(2).min(picker.faces.len().saturating_sub(VISIBLE));
                }
            }
        }
        Dialog {
            kind: Kind::Picker(Box::new(picker)),
            title,
            body: String::new(),
            width: 760.0,
            buttons: vec![
                (if reopen { "Another File\u{2026}".into() } else { "Cancel".into() }, false, DAct::PickerOther),
                ("Use this font".into(), true, DAct::PickerUse),
            ],
        }
    }

    // ---- what the tiles do ----

    fn message(&mut self, title: &str, body: &str) {
        self.dialog = Some(Dialog { kind: Kind::Message, title: title.into(), body: body.into(), width: 640.0, buttons: vec![("OK".into(), true, DAct::Close)] });
    }

    fn act(&mut self, act: Act) {
        match act {
            Act::Settings => {
                if !self.busy() {
                    self.dialog = Some(App::settings_dialog());
                }
            }
            Act::Choose(t) => self.choose(t),
            Act::Clear(t) => {
                if !self.busy() {
                    self.release(t);
                    self.converted = false;
                }
            }
            Act::PickFolder => self.pick_folder(),
            Act::ClearAll => {
                if !self.busy() {
                    for t in 0..self.tiles.len() {
                        self.release(t);
                    }
                    self.converted = false;
                }
            }
            Act::SetTarget(t) => self.set_target(t),
            Act::Convert => self.convert(),
            Act::OpenOutput => self.open_output(),
            Act::Button(i) => self.dialog_button(i),
            Act::Row(n) => {
                if let Some(Dialog { kind: Kind::Picker(picker), .. }) = self.dialog.as_mut() {
                    picker.selected = n;
                }
            }
            Act::TextSize(i) => self.set_look(i, self.theme.light),
            Act::Light(light) => {
                let size = places::TEXT_SCALES.iter().position(|s| (*s - self.theme.text_scale).abs() < 0.001).unwrap_or(1);
                self.set_look(size, light);
            }
        }
    }

    /// A text size or theme picked in Settings: kept, and shown at once.
    fn set_look(&mut self, size: usize, light: bool) {
        self.theme.text_scale = places::TEXT_SCALES[size];
        self.theme.light = light;
        places::set("textsize", places::TEXT_SIZES[size]);
        places::set("theme", if light { "light" } else { "dark" });
        self.ctx.send_viewport_cmd(ViewportCommand::SetTheme(if light { egui::SystemTheme::Light } else { egui::SystemTheme::Dark }));
    }

    fn dialog_button(&mut self, i: usize) {
        let Some(d) = &self.dialog else { return };
        let Some((_, _, act)) = d.buttons.get(i).cloned() else { return };
        match act {
            DAct::Close => self.dialog = None,
            DAct::Replace => {
                self.dialog = None;
                self.run_job();
            }
            DAct::PickerUse => {
                if let Some(Dialog { kind: Kind::Picker(picker), .. }) = self.dialog.take() {
                    let choice = picker.faces[picker.selected].clone();
                    self.apply(picker.tile, choice, picker.faces);
                }
            }
            DAct::PickerOther => {
                if let Some(Dialog { kind: Kind::Picker(picker), .. }) = self.dialog.take() {
                    if picker.reopen {
                        self.browse(picker.tile);
                    }
                }
            }
        }
    }

    /// Escape and Enter in a pop-up: the quiet button and the main one; the arrows in the
    /// picker.
    fn dialog_key(&mut self, key: Key) {
        let Some(d) = self.dialog.as_mut() else { return };
        if let Kind::Picker(picker) = &mut d.kind {
            match key {
                Key::Escape => {
                    self.dialog = None;
                    return;
                }
                Key::ArrowDown | Key::ArrowUp => {
                    let n = picker.faces.len();
                    picker.selected = if key == Key::ArrowDown { (picker.selected + 1).min(n - 1) } else { picker.selected.saturating_sub(1) };
                    if picker.selected < picker.top {
                        picker.top = picker.selected;
                    }
                    if picker.selected >= picker.top + VISIBLE {
                        picker.top = picker.selected + 1 - VISIBLE;
                    }
                    return;
                }
                _ => {}
            }
        }
        let primary = key == Key::Enter;
        if !primary && key != Key::Escape {
            return;
        }
        let i = d.buttons.iter().position(|b| b.1 == primary).unwrap_or(0);
        if !d.buttons.is_empty() {
            self.dialog_button(i);
        }
    }

    fn set_target(&mut self, value: Target) {
        if self.busy() || self.target == value {
            return;
        }
        self.target = value;
        places::save_target(value);
        for t in &mut self.tiles {
            t.state.converted = false;
        }
        self.converted = false;
    }

    fn open_output(&mut self) {
        if let Err(e) = std::fs::create_dir_all(&self.output) {
            self.message("The folder can't be opened", &e.to_string());
            return;
        }
        open_in_system(&self.output);
    }

    fn choose(&mut self, t: usize) {
        if self.busy() {
            return;
        }
        if let Some(faces) = self.tiles[t].state.file_faces.clone() {
            if faces.len() > 1 {
                self.dialog = Some(self.picker(t, faces, true));
                return;
            }
        }
        self.browse(t);
    }

    fn browse(&mut self, t: usize) {
        let mut dialog = rfd::FileDialog::new()
            .set_title(format!("Choose a font for {}", self.tiles[t].def.name))
            .add_filter("Fonts (*.ttf, *.otf, *.ttc, *.otc, *.woff, *.woff2)", &font::EXTENSIONS)
            .add_filter("All files", &["*"]);
        if let Some(folder) = &self.last_font_folder {
            dialog = dialog.set_directory(folder);
        }
        let Some(file) = dialog.pick_file() else { return };
        self.last_font_folder = file.parent().map(Path::to_path_buf);
        self.open(t, &file);
    }

    fn open(&mut self, t: usize, path: &Path) {
        if !font::accepted(path) {
            self.message("This font type can't be used", &not_a_font_type(path));
            return;
        }
        let faces = match font::faces(path) {
            Ok(f) => f,
            Err(e) => {
                self.message("This font can't be read", &e.0);
                return;
            }
        };
        if faces.len() == 1 {
            let choice = faces[0].clone();
            self.apply(t, choice, faces);
        } else {
            self.dialog = Some(self.picker(t, faces, false));
        }
    }

    fn apply(&mut self, t: usize, choice: FontFaceChoice, faces: Vec<FontFaceChoice>) {
        match checked_face(&choice) {
            Ok((face, missing)) => {
                self.release(t);
                self.tiles[t].state = TileState { choice: Some(choice), face: Some(face), file_faces: Some(faces), missing, converted: false };
                self.converted = false;
            }
            Err((title, body)) => self.message(&title, &body),
        }
    }

    fn release(&mut self, t: usize) {
        self.previews.retain(|k, _| k.0 != t);
        self.measures.retain(|k, _| k.0 != t);
        self.tiles[t].state = TileState::default();
    }

    fn pick_folder(&mut self) {
        if self.busy() {
            return;
        }
        let mut dialog = rfd::FileDialog::new().set_title("Where the game fonts go. The game's Override folder is where the game reads them.");
        if self.output.is_dir() {
            dialog = dialog.set_directory(&self.output);
        }
        let Some(folder) = dialog.pick_folder() else { return };
        if !same_path(&folder, &self.output) {
            for t in &mut self.tiles {
                t.state.converted = false; // nothing written there yet
            }
            self.converted = false;
        }
        self.output = folder;
        places::save_folder(&self.output);
    }

    fn convert(&mut self) {
        if self.busy() {
            return;
        }
        let chosen: Vec<usize> = (0..self.tiles.len()).filter(|&t| self.tiles[t].state.face.is_some()).collect();
        if chosen.is_empty() {
            return;
        }
        let names: Vec<&str> = chosen.iter().flat_map(|&t| self.tiles[t].def.resrefs.iter().copied()).collect();
        let existing = conversion::existing(&self.output, names);
        if existing > 0 {
            let body = format!(
                "The folder already has {existing} font file{} with the names these fonts use. Converting replaces the .tga and .txi files. A .tpc file is kept, renamed to end in {}.",
                if existing == 1 { "" } else { "s" },
                conversion::TPC_BACKUP
            );
            self.dialog = Some(Dialog {
                kind: Kind::Message,
                title: "Replace fonts already there?".into(),
                body,
                width: 640.0,
                buttons: vec![("Cancel".into(), false, DAct::Close), ("Replace".into(), true, DAct::Replace)],
            });
            return;
        }
        self.run_job();
    }

    fn run_job(&mut self) {
        let chosen: Vec<usize> = (0..self.tiles.len()).filter(|&t| self.tiles[t].state.face.is_some()).collect();
        let work: Vec<(String, Arc<LoadedFace>)> = chosen
            .iter()
            .flat_map(|&t| self.tiles[t].def.resrefs.iter().map(move |r| (t, r)))
            .map(|(t, r)| (r.to_string(), self.tiles[t].state.face.clone().unwrap()))
            .collect();
        let progress = Arc::new(AtomicUsize::new(0));
        let stop = Arc::new(AtomicBool::new(false));
        let result = Arc::new(Mutex::new(None));
        let total = work.len();
        let mut job = Conversion {
            folder: self.output.clone(),
            target: self.target,
            references: self.references.clone(),
            work,
            progress: progress.clone(),
            stop: stop.clone(),
            moving: false,
        };
        let (ctx, out) = (self.ctx.clone(), result.clone());
        std::thread::spawn(move || {
            let r = job.run(&|| ctx.request_repaint());
            *out.lock().unwrap() = Some((r, job.moving));
            ctx.request_repaint();
        });
        self.job = Some(Job { progress, total, stop, result, tiles: chosen });
    }

    /// The conversion's end, on the window's thread.
    fn poll_job(&mut self) {
        let Some(job) = &self.job else { return };
        let Some((result, moving)) = job.result.lock().unwrap().take() else { return };
        let tiles = job.tiles.clone();
        self.job = None;
        if self.close_when_done {
            self.ctx.send_viewport_cmd(ViewportCommand::Close);
            return;
        }
        match result {
            Ok(()) => {
                // Done: Convert turns green ("Converted"), no pop-up.
                for t in tiles {
                    self.tiles[t].state.converted = true;
                }
                self.converted = true;
            }
            Err(Failure::Stopped) => self.exit_code.store(1, Ordering::SeqCst),
            Err(Failure::Failed(m)) => {
                self.exit_code.store(1, Ordering::SeqCst);
                self.message("The fonts could not be written", &write_failure(&m, moving));
            }
        }
    }

    fn hit_at(&self, at: Pos2) -> Option<&Hit> {
        let c = pos2((at.x - self.ox) / self.scale, (at.y - self.oy) / self.scale);
        self.hits.iter().rev().find(|h| h.rect.contains(c))
    }

    fn tile_at(&self, at: Pos2) -> Option<usize> {
        let c = pos2((at.x - self.ox) / self.scale, (at.y - self.oy) / self.scale);
        // A tile's bounds are where its hits are; the whole tile takes a drop.
        let mut found = None;
        let rows: [&[&str]; 2] = [&["Menus"], &["Reading", "HUD", "Credits"]];
        let mut y = 146.0;
        for row in rows {
            let count: usize = row.iter().map(|n| self.tiles.iter().filter(|t| t.def.group == *n).count()).sum();
            let available = 1480.0 - 14.0 * (row.len() as f32 - 1.0);
            let mut x = 60.0;
            for name in row {
                let members: Vec<usize> = (0..self.tiles.len()).filter(|&i| self.tiles[i].def.group == *name).collect();
                let w = available * members.len() as f32 / count as f32;
                let n = members.len() as f32;
                let tw = (w - 24.0 - 12.0 * (n - 1.0)) / n;
                for (i, &t) in members.iter().enumerate() {
                    let r = Rect::from_min_size(pos2(x + 12.0 + i as f32 * (tw + 12.0), y + 43.0), vec2(tw, 236.0));
                    if r.contains(c) {
                        found = Some(t);
                    }
                }
                x += w + 14.0;
            }
            y += 291.0 + 14.0;
        }
        found
    }

    fn first_frame(&mut self, ctx: &egui::Context) {
        let monitor = ctx.input(|i| i.viewport().monitor_size);
        self.monitor = monitor;
        if let Some((w, h)) = self.size_arg {
            let ppp = ctx.pixels_per_point();
            ctx.send_viewport_cmd(ViewportCommand::InnerSize(vec2(w as f32 / ppp, h as f32 / ppp)));
        } else if let Some(m) = monitor {
            let size = self.fit_window(ctx, m);
            ctx.send_viewport_cmd(ViewportCommand::InnerSize(size));
            ctx.send_viewport_cmd(ViewportCommand::OuterPosition(pos2(((m.x - size.x) / 2.0).max(0.0), ((m.y - size.y) / 2.0 - 16.0).max(0.0))));
        }
        if self.convert_on_show {
            self.convert();
        }
    }

    /// The window fitted to a screen, as KMSE fits its own (2026-11-09, the maintainer: "make
    /// it dynamic, like KMSE smaller on smaller screens"): 1600x900 at the screen's scale
    /// where that fits 94% of its width and 88% of its height, else as large as does; the
    /// whole canvas shrinks evenly with the window, never below half its size (KMSE's floor),
    /// and never with a smallest size larger than the screen. The size it opens at.
    fn fit_window(&self, ctx: &egui::Context, m: egui::Vec2) -> egui::Vec2 {
        let fit = 1f32.min((m.x * 0.94 / CW).min(m.y * 0.88 / CH));
        let least = 0.5f32.min(fit);
        ctx.send_viewport_cmd(ViewportCommand::MinInnerSize(vec2(CW * least, CH * least)));
        vec2(CW * fit, CH * fit)
    }

    /// Moved to another screen (or the screen's scale changed): the smallest size follows it,
    /// and a window larger than the new screen is made to fit it.
    fn follow_monitor(&mut self, ctx: &egui::Context) {
        let Some(m) = ctx.input(|i| i.viewport().monitor_size) else { return };
        if self.monitor.is_some_and(|old| (old - m).length() < 1.0) {
            return;
        }
        self.monitor = Some(m);
        let size = self.fit_window(ctx, m);
        let inner = ctx.content_rect().size();
        if inner.x > m.x * 0.98 || inner.y > m.y * 0.94 {
            ctx.send_viewport_cmd(ViewportCommand::InnerSize(size));
        }
    }
}

impl eframe::App for App {
    fn clear_color(&self, _visuals: &egui::Visuals) -> [f32; 4] {
        self.theme.window().to_normalized_gamma_f32()
    }

    fn ui(&mut self, ui: &mut egui::Ui, _frame: &mut eframe::Frame) {
        let ctx = ui.ctx().clone();
        if self.first_frame {
            self.first_frame = false;
            self.first_frame(&ctx);
        } else if self.size_arg.is_none() {
            self.follow_monitor(&ctx);
        }
        self.poll_job();

        // Closing while fonts are being made: the conversion is stopped, nothing it made is
        // put in the folder, and the window closes when it has stopped (audit F3).
        if ctx.input(|i| i.viewport().close_requested()) && self.busy() {
            ctx.send_viewport_cmd(ViewportCommand::CancelClose);
            if let Some(job) = &self.job {
                job.stop.store(true, Ordering::SeqCst);
            }
            self.close_when_done = true;
        }

        self.view = ctx.content_rect();
        self.ppp = ctx.pixels_per_point();
        self.scale = (self.view.width() / CW).min(self.view.height() / CH);
        self.ox = self.snap(self.view.min.x + (self.view.width() - CW * self.scale) / 2.0);
        self.oy = self.snap(self.view.min.y + (self.view.height() - CH * self.scale) / 2.0);

        // A font file dragged over a tile lights it; dropped, it is opened for that tile.
        let (hovering, dropped, pointer) = ctx.input(|i| {
            (!i.raw.hovered_files.is_empty(), i.raw.dropped_files.first().map(|f| f.path().to_path_buf()), i.pointer.latest_pos())
        });
        if let Some(path) = dropped {
            self.pending_drop = Some(path);
        }
        let free = self.dialog.is_none() && !self.busy();
        self.drop_target = if (hovering || self.pending_drop.is_some()) && free { pointer.and_then(|p| self.tile_at(p)) } else { None };
        if let Some(path) = self.pending_drop.take() {
            match (free, self.drop_target) {
                (true, Some(t)) => {
                    self.drop_target = None;
                    self.open(t, &path);
                }
                // Where the system gave no place for the drop yet, it waits for the pointer.
                (true, None) if pointer.is_none() => self.pending_drop = Some(path),
                _ => {}
            }
        }

        // Keys in a pop-up.
        if self.dialog.is_some() {
            for key in [Key::Escape, Key::Enter, Key::ArrowUp, Key::ArrowDown] {
                if ctx.input(|i| i.key_pressed(key)) {
                    self.dialog_key(key);
                }
            }
        }

        // The pointer, against what the last frame painted.
        if !self.still {
            let (hover_pos, pressed, released, double) = ctx.input(|i| {
                (i.pointer.hover_pos(), i.pointer.primary_pressed(), i.pointer.primary_released(), i.pointer.button_double_clicked(egui::PointerButton::Primary))
            });
            let hit = hover_pos.and_then(|p| self.hit_at(p)).map(|h| (h.id.clone(), h.act));
            self.hover = hit.as_ref().map(|h| h.0.clone());
            if hit.is_some() {
                ctx.set_cursor_icon(CursorIcon::PointingHand);
            }
            if pressed {
                self.pressed = hit.as_ref().map(|h| h.0.clone());
                if double {
                    if let (Some((id, Act::Row(_))), Some(Dialog { kind: Kind::Picker(_), .. })) = (&hit, &self.dialog) {
                        if id.starts_with("row") {
                            self.dialog_button(1);
                        }
                    }
                }
            }
            if released {
                if let Some((id, act)) = hit {
                    if self.pressed.as_deref() == Some(id.as_str()) {
                        self.act(act);
                    }
                }
                self.pressed = None;
            }
            let scroll = ctx.input(|i| i.smooth_scroll_delta.y);
            if let Some(Dialog { kind: Kind::Picker(picker), .. }) = self.dialog.as_mut() {
                self.scroll += scroll;
                while self.scroll.abs() >= 40.0 {
                    let up = self.scroll > 0.0;
                    self.scroll -= 40f32.copysign(self.scroll);
                    let max = picker.faces.len().saturating_sub(VISIBLE);
                    picker.top = if up { picker.top.saturating_sub(1) } else { (picker.top + 1).min(max) };
                }
            } else {
                self.scroll = 0.0;
            }
        }

        let painter = ctx.layer_painter(egui::LayerId::background());
        self.paint(&painter);
        if self.busy() {
            ctx.request_repaint_after(std::time::Duration::from_millis(100));
        }
    }
}

// ---- helpers ----

/// What KMRP's HUD font's black ring is at this height: one original pixel, a sixth of its
/// 6x atlas's 96 pixel glyph box per 16.
fn ring_for(piece: &Piece, h: f32) -> f32 {
    atlas::outlined(&piece.resref).map(|r| h * r / 96.0).unwrap_or(0.0)
}

fn lerp(a: Color32, b: Color32, t: f32) -> Color32 {
    let m = |x: u8, y: u8| (x as f32 + (y as f32 - x as f32) * t).round() as u8;
    Color32::from_rgba_premultiplied(m(a.r(), b.r()), m(a.g(), b.g()), m(a.b(), b.b()), m(a.a(), b.a()))
}

/// A convex polygon filled with a vertical gradient from `top` at y0 to `bottom` at y1.
fn fill_gradient(p: &egui::Painter, pts: &[Pos2], y0: f32, y1: f32, top: Color32, bottom: Color32) {
    if pts.len() < 3 {
        return;
    }
    let colour = |y: f32| lerp(top, bottom, ((y - y0) / (y1 - y0).max(0.001)).clamp(0.0, 1.0));
    let mut mesh = Mesh::default();
    let c = pts.iter().fold(vec2(0.0, 0.0), |a, q| a + q.to_vec2()) / pts.len() as f32;
    mesh.colored_vertex(c.to_pos2(), colour(c.y));
    for q in pts {
        mesh.colored_vertex(*q, colour(q.y));
    }
    let n = pts.len() as u32;
    for i in 0..n {
        mesh.add_triangle(0, 1 + i, 1 + (i + 1) % n);
    }
    p.add(Shape::mesh(mesh));
}

/// A convex polygon cut at x = `cut`, keeping the left.
fn clip_right(pts: &[Pos2], cut: f32) -> Vec<Pos2> {
    clip(pts, |q| q.x <= cut, |a, b| {
        let t = (cut - a.x) / (b.x - a.x);
        pos2(cut, a.y + (b.y - a.y) * t)
    })
}

/// A convex polygon cut at y = `cut`, keeping the top.
fn clip_below(pts: &[Pos2], cut: f32) -> Vec<Pos2> {
    clip(pts, |q| q.y <= cut, |a, b| {
        let t = (cut - a.y) / (b.y - a.y);
        pos2(a.x + (b.x - a.x) * t, cut)
    })
}

fn clip(pts: &[Pos2], inside: impl Fn(Pos2) -> bool, cross: impl Fn(Pos2, Pos2) -> Pos2) -> Vec<Pos2> {
    let mut out = Vec::new();
    for i in 0..pts.len() {
        let (a, b) = (pts[i], pts[(i + 1) % pts.len()]);
        match (inside(a), inside(b)) {
            (true, true) => out.push(b),
            (true, false) => out.push(cross(a, b)),
            (false, true) => {
                out.push(cross(a, b));
                out.push(b);
            }
            (false, false) => {}
        }
    }
    out
}

fn same_path(a: &Path, b: &Path) -> bool {
    if cfg!(any(windows, target_os = "macos")) {
        a.to_string_lossy().eq_ignore_ascii_case(&b.to_string_lossy())
    } else {
        a == b
    }
}

/// The folder in the system's file manager.
fn open_in_system(path: &Path) {
    let program = if cfg!(windows) {
        "explorer"
    } else if cfg!(target_os = "macos") {
        "open"
    } else {
        "xdg-open"
    };
    let _ = std::process::Command::new(program).arg(path).spawn();
}
