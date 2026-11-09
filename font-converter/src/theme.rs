// KMRP Font Converter: the colours and type of the KMRP installer.
//
// Dark is the installer's; light is the Save Editor's light mode. Edges were raised on
// 2026-10-22 to at least 1.8:1 against the fill they sit on (WCAG contrast), both modes.
// The values are Theme.cs's, unchanged by the move to Rust.

use eframe::egui::{self, Color32, FontFamily, FontId};

#[derive(Clone, Copy)]
pub struct Theme {
    pub light: bool,
    /// Text size: KMSE's steps 1, 1.2 (Medium, the default) and 1.32.
    pub text_scale: f32,
}

const fn rgb(r: u8, g: u8, b: u8) -> Color32 {
    Color32::from_rgb(r, g, b)
}

pub fn alpha(c: Color32, a: u8) -> Color32 {
    Color32::from_rgba_unmultiplied(c.r(), c.g(), c.b(), a)
}

pub const ACCENT_TOP: Color32 = rgb(47, 208, 245);
pub const ACCENT_BOTTOM: Color32 = rgb(26, 179, 227);
pub const ACCENT_HOVER: Color32 = rgb(88, 222, 250);
pub const ACCENT_INK: Color32 = rgb(6, 18, 30);
// The examples are the game's own coloured text, so their backing stays dark in both.
pub const EXAMPLE: Color32 = rgb(5, 10, 18);

macro_rules! pick {
    ($name:ident, $r:expr, $g:expr, $b:expr, $lr:expr, $lg:expr, $lb:expr) => {
        pub fn $name(&self) -> Color32 {
            if self.light {
                rgb($lr, $lg, $lb)
            } else {
                rgb($r, $g, $b)
            }
        }
    };
}

impl Theme {
    pick!(window, 10, 18, 32, 232, 238, 245);
    pick!(header_glow, 22, 36, 58, 211, 224, 238);
    pick!(group, 11, 19, 34, 248, 250, 253);
    pick!(group_edge, 48, 70, 102, 163, 180, 200);
    pick!(tile, 15, 26, 44, 255, 255, 255);
    pick!(tile_hover, 19, 32, 53, 237, 243, 249);
    pick!(tile_edge, 62, 88, 126, 147, 166, 188);
    pick!(example_edge, 46, 66, 96, 46, 66, 96);
    pick!(field, 6, 11, 20, 255, 255, 255);
    pick!(edge, 68, 96, 138, 130, 150, 173);
    pick!(button, 11, 20, 36, 255, 255, 255);
    pick!(button_hover, 20, 34, 56, 237, 243, 249);
    pick!(panel, 13, 22, 38, 248, 250, 253);
    pick!(selected, 11, 28, 44, 220, 236, 248);
    pick!(accent, 42, 198, 239, 10, 127, 176);
    pick!(gold, 226, 195, 92, 122, 95, 8);
    pick!(picked, 255, 154, 60, 199, 95, 0);
    pick!(done, 126, 224, 171, 30, 138, 82);
    pick!(text, 236, 243, 248, 13, 26, 43);
    pick!(muted, 177, 195, 208, 68, 87, 108);
    pick!(soft, 158, 177, 194, 93, 112, 134);
    pick!(faint, 138, 155, 172, 86, 104, 124);
    pick!(warning, 255, 207, 112, 160, 100, 0);
    pick!(dashed, 74, 108, 156, 125, 147, 173);
    pick!(choose_ink, 159, 216, 243, 10, 110, 160);
    pick!(radio, 74, 102, 140, 130, 150, 173);
    // KOTOR's own GUI blue (vanilla .gui TEXT COLOR 0, 168, 250): the Version box's rings.
    pick!(kotor_blue, 0, 168, 250, 0, 120, 190);

    /// Over the window behind a pop-up.
    pub fn dim(&self) -> Color32 {
        if self.light {
            Color32::from_rgba_unmultiplied(26, 42, 58, 102)
        } else {
            Color32::from_rgba_unmultiplied(3, 6, 12, 184)
        }
    }

    /// The pop-up's own shadow.
    pub fn shadow(&self) -> Color32 {
        if self.light {
            Color32::from_rgba_unmultiplied(26, 42, 58, 46)
        } else {
            Color32::from_rgba_unmultiplied(0, 0, 0, 110)
        }
    }

    /// A length for text drawn at `px`, at the text size in use.
    pub fn t(&self, px: f32) -> f32 {
        px * self.text_scale
    }
}

/// Which face a piece of the window's text is set in.
#[derive(Clone, Copy, PartialEq)]
pub enum Face {
    /// KMRP Original HD Menu: titles and buttons.
    Display,
    /// KMRP Original HD Descriptions: text.
    Body,
    /// The folder's path.
    Mono,
}

/// A face at a size in canvas pixels, before the text scale.
#[derive(Clone, Copy)]
pub struct Font {
    pub face: Face,
    pub px: f32,
}

pub fn display(px: f32) -> Font {
    Font { face: Face::Display, px }
}

pub fn body(px: f32) -> Font {
    Font { face: Face::Body, px }
}

pub fn mono(px: f32) -> Font {
    Font { face: Face::Mono, px }
}

impl Font {
    /// The egui font at `size` points.
    pub fn id(&self, size: f32) -> FontId {
        let family = match self.face {
            Face::Display => FontFamily::Name("display".into()),
            Face::Body => FontFamily::Name("body".into()),
            Face::Mono => FontFamily::Monospace,
        };
        // Quarter points: few sizes kept while the window is dragged to a new size.
        FontId::new(((size * 4.0).round() / 4.0).max(1.0), family)
    }
}

/// KMRP's own fonts for the window's text (fonts/*.ttf, SIL OFL 1.1), carried in the program,
/// with egui's own fonts after them for the few signs they lack (the warning sign, the
/// cross).
pub fn install_fonts(ctx: &egui::Context) {
    let mut fonts = egui::FontDefinitions::default();
    fonts.font_data.insert("kmrp-menu".into(), egui::FontData::from_static(crate::MENU_FONT).into());
    fonts.font_data.insert("kmrp-body".into(), egui::FontData::from_static(crate::BODY_FONT).into());
    let fallback: Vec<String> = fonts.families.get(&FontFamily::Proportional).cloned().unwrap_or_default();
    let mut display = vec!["kmrp-menu".to_string()];
    display.extend(fallback.iter().cloned());
    let mut text = vec!["kmrp-body".to_string()];
    text.extend(fallback.iter().cloned());
    fonts.families.insert(FontFamily::Name("display".into()), display);
    fonts.families.insert(FontFamily::Name("body".into()), text);
    ctx.set_fonts(fonts);
}
