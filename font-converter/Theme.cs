// KMRP Font Converter: the colours and type of the KMRP installer, and how text is drawn.
using System;
using System.Collections.Generic;
using System.Drawing;
using System.Drawing.Drawing2D;
using System.Drawing.Text;
using System.IO;
using System.Linq;
using System.Reflection;
using System.Runtime.InteropServices;

namespace KmrpFontConverter
{
    internal static class Theme
    {
        // Dark is the installer's; light is the Save Editor's light mode (save-editor/ui/
        // theme.slint). Edges were raised on 2026-10-22 to at least 1.8:1 against the fill they
        // sit on (WCAG contrast), both modes: the old dark ones measured 1.2 to 1.7.
        public static bool Light;
        /// <summary>Text size: KMSE's steps (its ui/theme.slint `ts`), 1, 1.2 (Medium, the
        /// default) and 1.32 (2026-11-08, the maintainer: "the same crispy and large fonts"
        /// as KMSE). Until then 1, 1.15 and 1.3.</summary>
        public static float TextScale = 1.2f;

        private static Color Pick(int r, int g, int b, int lr, int lg, int lb)
        {
            return Light ? Color.FromArgb(lr, lg, lb) : Color.FromArgb(r, g, b);
        }

        public static Color Window { get { return Pick(10, 18, 32, 232, 238, 245); } }
        public static Color HeaderGlow { get { return Pick(22, 36, 58, 211, 224, 238); } }
        public static Color Group { get { return Pick(11, 19, 34, 248, 250, 253); } }
        public static Color GroupEdge { get { return Pick(48, 70, 102, 163, 180, 200); } }
        public static Color Tile { get { return Pick(15, 26, 44, 255, 255, 255); } }
        public static Color TileHover { get { return Pick(19, 32, 53, 237, 243, 249); } }
        public static Color TileEdge { get { return Pick(62, 88, 126, 147, 166, 188); } }
        // The examples are the game's own coloured text, so their backing stays dark in both.
        public static Color Example { get { return Color.FromArgb(5, 10, 18); } }
        public static Color ExampleEdge { get { return Pick(46, 66, 96, 46, 66, 96); } }
        public static Color Field { get { return Pick(6, 11, 20, 255, 255, 255); } }
        public static Color Edge { get { return Pick(68, 96, 138, 130, 150, 173); } }
        public static Color Button { get { return Pick(11, 20, 36, 255, 255, 255); } }
        public static Color ButtonHover { get { return Pick(20, 34, 56, 237, 243, 249); } }
        public static Color Panel { get { return Pick(13, 22, 38, 248, 250, 253); } }
        public static Color Selected { get { return Pick(11, 28, 44, 220, 236, 248); } }
        public static Color Accent { get { return Pick(42, 198, 239, 10, 127, 176); } }
        public static readonly Color AccentTop = Color.FromArgb(47, 208, 245);
        public static readonly Color AccentBottom = Color.FromArgb(26, 179, 227);
        public static readonly Color AccentHover = Color.FromArgb(88, 222, 250);
        public static readonly Color AccentInk = Color.FromArgb(6, 18, 30);
        public static Color Gold { get { return Pick(226, 195, 92, 122, 95, 8); } }
        public static Color Picked { get { return Pick(255, 154, 60, 199, 95, 0); } }   // a tile with a font picked
        public static Color Done { get { return Pick(126, 224, 171, 30, 138, 82); } }   // and converted
        public static Color Text { get { return Pick(236, 243, 248, 13, 26, 43); } }
        public static Color Muted { get { return Pick(177, 195, 208, 68, 87, 108); } }
        public static Color Soft { get { return Pick(158, 177, 194, 93, 112, 134); } }
        // Raised on 2026-11-08 (122,138,154 and 143,163,181 before), as KMSE's quieter text is
        // nearer white: small labels read at the larger sizes without looking switched off.
        public static Color Faint { get { return Pick(138, 155, 172, 86, 104, 124); } }
        public static Color Success { get { return Pick(126, 224, 171, 30, 138, 82); } }
        public static Color Warning { get { return Pick(255, 207, 112, 160, 100, 0); } }
        public static Color Dashed { get { return Pick(74, 108, 156, 125, 147, 173); } }
        public static Color ChooseInk { get { return Pick(159, 216, 243, 10, 110, 160); } }
        public static Color Radio { get { return Pick(74, 102, 140, 130, 150, 173); } }
        /// <summary>KOTOR's own GUI blue (vanilla .gui TEXT COLOR 0, 168, 250): the Version
        /// box's radio rings, as KMRP's Text Size and Font page draws them in the game.</summary>
        public static Color KotorBlue { get { return Pick(0, 168, 250, 0, 120, 190); } }
        /// <summary>Over the window behind a pop-up, and the pop-up's own shadow.</summary>
        public static Color Dim { get { return Light ? Color.FromArgb(102, 26, 42, 58) : Color.FromArgb(184, 3, 6, 12); } }
        public static Color Shadow { get { return Light ? Color.FromArgb(46, 26, 42, 58) : Color.FromArgb(110, 0, 0, 0); } }

        /// <summary>A box's height for a line of text drawn at `px`, at the text size in use.</summary>
        public static float T(float px) { return px * TextScale; }

        private static readonly Dictionary<string, Font> fonts = new Dictionary<string, Font>();
        private static string display, body, mono;
        private static HashSet<string> installed;

        // The installer's faces, and what stands in for them where Windows' own fonts are
        // missing (Wine under Proton has neither Bahnschrift nor Segoe UI). The installed
        // families are listed once (audit: they were listed on every paint).
        private static string Family(params string[] names)
        {
            if (installed == null)
                installed = new HashSet<string>(FontFamily.Families.Select(f => f.Name), StringComparer.OrdinalIgnoreCase);
            foreach (string name in names) if (installed.Contains(name)) return name;
            return FontFamily.GenericSansSerif.Name;
        }

        // KMRP's own fonts, carried in the program (fonts/*.ttf, SIL OFL 1.1): Menu for titles,
        // Descriptions for text, as the Save Editor. Loaded from memory, so nothing is
        // installed; the system's faces stand in if they cannot be loaded.
        private static readonly PrivateFontCollection kmrp = new PrivateFontCollection();
        private static Dictionary<string, FontFamily> kmrpFamilies;

        private static FontFamily Kmrp(string name)
        {
            if (kmrpFamilies == null)
            {
                Assembly asm = Assembly.GetExecutingAssembly();
                foreach (string res in asm.GetManifestResourceNames())
                {
                    if (!res.StartsWith("fonts.") || !res.EndsWith(".ttf")) continue;
                    using (Stream s = asm.GetManifestResourceStream(res))
                    {
                        var data = new byte[s.Length];
                        s.Read(data, 0, data.Length);
                        // GDI+ reads the font from this memory for as long as the program runs.
                        IntPtr p = Marshal.AllocCoTaskMem(data.Length);
                        Marshal.Copy(data, 0, p, data.Length);
                        try { kmrp.AddMemoryFont(p, data.Length); } catch (Exception) { }
                    }
                }
                kmrpFamilies = kmrp.Families.ToDictionary(f => f.Name);
            }
            FontFamily family;
            return kmrpFamilies.TryGetValue(name, out family) ? family : null;
        }

        public static Font Display(float px) { return DisplayAt(px * TextScale); }

        private static Font DisplayAt(float px)
        {
            FontFamily f = Kmrp("KMRP Original HD Menu");
            if (f != null) return Get(f, px, FontStyle.Regular);
            return Get(display ?? (display = Family("Bahnschrift SemiCondensed", "Bahnschrift", "Segoe UI Semibold", "Tahoma", "Arial")), px, FontStyle.Bold);
        }

        public static Font Body(float px, bool bold = false) { return BodyAt(px * TextScale, bold); }

        private static Font BodyAt(float px, bool bold)
        {
            // Descriptions is the game's bold body face already; one weight serves both.
            FontFamily f = Kmrp("KMRP Original HD Descriptions");
            if (f != null) return Get(f, px, FontStyle.Regular);
            return Get(body ?? (body = Family("Segoe UI", "Tahoma", "Arial")), px, bold ? FontStyle.Bold : FontStyle.Regular);
        }

        public static Font Mono(float px) { return Get(mono ?? (mono = Family("Consolas", "Courier New")), px * TextScale, FontStyle.Regular); }

        private static Font Get(string family, float px, FontStyle style)
        {
            string key = family + "|" + px + "|" + style;
            Font font;
            if (!fonts.TryGetValue(key, out font)) fonts[key] = font = new Font(family, px, style, GraphicsUnit.Pixel);
            return font;
        }

        private static Font Get(FontFamily family, float px, FontStyle style)
        {
            string key = "kmrp:" + family.Name + "|" + px + "|" + style;
            Font font;
            if (!fonts.TryGetValue(key, out font)) fonts[key] = font = new Font(family, px, style, GraphicsUnit.Pixel);
            return font;
        }

        /// <summary>The same face at `factor` times its size, for drawing at device pixels.</summary>
        private static Font Scaled(Font font, float factor)
        {
            if (Math.Abs(factor - 1) < 0.0001f) return font;
            float px = (float)Math.Round(font.Size * factor * 4) / 4;   // quarter pixels: few fonts kept
            string key = "scaled:" + font.FontFamily.Name + "|" + px + "|" + font.Style;
            Font scaled;
            if (!fonts.TryGetValue(key, out scaled)) fonts[key] = scaled = new Font(font.FontFamily, px, font.Style, GraphicsUnit.Pixel);
            return scaled;
        }

        public static GraphicsPath Round(RectangleF r, float radius)
        {
            var path = new GraphicsPath();
            float d = Math.Min(radius * 2, Math.Min(r.Width, r.Height));
            if (d <= 0) { path.AddRectangle(r); return path; }
            path.AddArc(r.X, r.Y, d, d, 180, 90);
            path.AddArc(r.Right - d, r.Y, d, d, 270, 90);
            path.AddArc(r.Right - d, r.Bottom - d, d, d, 0, 90);
            path.AddArc(r.X, r.Bottom - d, d, d, 90, 90);
            path.CloseFigure();
            return path;
        }

        public static void Box(Graphics g, RectangleF r, float radius, Color fill, Color edge, float width = 1)
        {
            using (var path = Round(r, radius))
            {
                using (var brush = new SolidBrush(fill)) g.FillPath(brush, path);
                if (edge.A > 0) using (var pen = new Pen(edge, width)) g.DrawPath(pen, path);
            }
        }

        public static readonly StringFormat Left = new StringFormat(StringFormat.GenericTypographic)
        { LineAlignment = StringAlignment.Center, Trimming = StringTrimming.EllipsisCharacter, FormatFlags = StringFormatFlags.NoWrap | StringFormatFlags.MeasureTrailingSpaces | StringFormatFlags.NoClip };
        public static readonly StringFormat Centre = new StringFormat(StringFormat.GenericTypographic)
        { Alignment = StringAlignment.Center, LineAlignment = StringAlignment.Center, Trimming = StringTrimming.EllipsisCharacter, FormatFlags = StringFormatFlags.NoWrap | StringFormatFlags.NoClip };
        public static readonly StringFormat Path = new StringFormat(StringFormat.GenericTypographic)
        { LineAlignment = StringAlignment.Center, Trimming = StringTrimming.EllipsisPath, FormatFlags = StringFormatFlags.NoWrap | StringFormatFlags.NoClip };
        public static readonly StringFormat Wrap = new StringFormat(StringFormat.GenericTypographic) { Trimming = StringTrimming.Word };

        // ---- text at the device's own pixels ----
        //
        // The window is laid out on a 1600x900 canvas and scaled to the window by the
        // graphics' transform. Text drawn through that transform was hinted at the canvas
        // size and then scaled, and its lines stood wherever the layout's arithmetic put
        // them, between pixels, which left it soft (2026-11-08, measured against KMSE, which
        // draws at the window's own pixels). Every piece of text is now drawn with the
        // transform taken off: the font made at its size on the screen, and the line put on a
        // whole pixel.

        private struct Device
        {
            public float Scale, X, Y;
            public float Px(float canvasX) { return canvasX * Scale + X; }
            public float Py(float canvasY) { return canvasY * Scale + Y; }
        }

        private static Device Take(Graphics g)
        {
            float[] m;
            using (Matrix t = g.Transform) m = t.Elements;
            g.ResetTransform();
            return new Device { Scale = m[0], X = m[4], Y = m[5] };
        }

        private static StringFormat Near(StringFormat format)
        {
            var near = (StringFormat)format.Clone();
            near.Alignment = StringAlignment.Near;
            near.LineAlignment = StringAlignment.Near;
            return near;
        }

        private static readonly Dictionary<StringFormat, StringFormat> nearFormats = new Dictionary<StringFormat, StringFormat>();

        /// <summary>One line of text in `r` (canvas units), aligned by `format`, on whole pixels.</summary>
        public static void Draw(Graphics g, string text, Font font, Color colour, RectangleF r, StringFormat format = null)
        {
            format = format ?? Left;
            GraphicsState state = g.Save();
            try
            {
                Device d = Take(g);
                Font f = Scaled(font, d.Scale);
                float x = d.Px(r.X), y = d.Py(r.Y), w = r.Width * d.Scale, h = r.Height * d.Scale;
                float lineH = f.Size * f.FontFamily.GetLineSpacing(f.Style) / f.FontFamily.GetEmHeight(f.Style);
                if (format.LineAlignment == StringAlignment.Center) y += (h - lineH) / 2;
                else if (format.LineAlignment == StringAlignment.Far) y += h - lineH;
                if (format.Alignment != StringAlignment.Near)
                {
                    float tw = g.MeasureString(text, f, PointF.Empty, StringFormat.GenericTypographic).Width;
                    if (tw < w) { x += format.Alignment == StringAlignment.Center ? (w - tw) / 2 : w - tw; w = tw + 2; }
                }
                StringFormat near;
                if (!nearFormats.TryGetValue(format, out near)) nearFormats[format] = near = Near(format);
                using (var brush = new SolidBrush(colour))
                    g.DrawString(text, f, brush, new RectangleF((float)Math.Round(x), (float)Math.Round(y), w, lineH + 2), near);
            }
            finally { g.Restore(state); }
        }

        /// <summary>Text whose line starts at canvas point (x, y), as DrawString with
        /// GenericTypographic puts it, on whole pixels.</summary>
        public static void DrawAt(Graphics g, string text, Font font, Color colour, float x, float y)
        {
            GraphicsState state = g.Save();
            try
            {
                Device d = Take(g);
                using (var brush = new SolidBrush(colour))
                    g.DrawString(text, Scaled(font, d.Scale), brush, (float)Math.Round(d.Px(x)), (float)Math.Round(d.Py(y)), StringFormat.GenericTypographic);
            }
            finally { g.Restore(state); }
        }

        /// <summary>Wrapped text in `r` (canvas units), its first line on a whole pixel.</summary>
        public static void DrawWrapped(Graphics g, string text, Font font, Color colour, RectangleF r)
        {
            GraphicsState state = g.Save();
            try
            {
                Device d = Take(g);
                var device = new RectangleF((float)Math.Round(d.Px(r.X)), (float)Math.Round(d.Py(r.Y)), r.Width * d.Scale, r.Height * d.Scale);
                using (var brush = new SolidBrush(colour)) g.DrawString(text, Scaled(font, d.Scale), brush, device, Wrap);
            }
            finally { g.Restore(state); }
        }

        /// <summary>Where a capital's ink starts and ends below the top of the line drawn with
        /// StringFormat.GenericTypographic, in pixels: measured from an "H".</summary>
        public static void Caps(Font font, out float top, out float bottom)
        {
            using (var path = new GraphicsPath())
            {
                path.AddString("H", font.FontFamily, (int)font.Style, font.Size, PointF.Empty, StringFormat.GenericTypographic);
                RectangleF ink = path.GetBounds();
                top = ink.Top;
                bottom = ink.Bottom;
            }
        }

        public static float Width(Graphics g, string text, Font font)
        {
            return g.MeasureString(text, font, PointF.Empty, StringFormat.GenericTypographic).Width;
        }
    }
}
