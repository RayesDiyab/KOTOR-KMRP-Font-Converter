// KMRP Font Converter: the window.
//
// One window, painted by hand in the KMRP installer's colours: eleven tiles, one for each
// distinct font the game draws its text in, grouped as the game uses them. A tile shows its
// text in KMRP's Original HD Font until the player picks a font for it, then in that font.
// Convert writes the picked fonts' atlases (Conversion.cs, Atlas.cs) into a folder, for the
// version chosen in the Version box beside it: KMRP, which sizes them to the screen, or the
// original game, at the game's own font sizes (2026-11-08). With the original game chosen
// the logo's word is KOTOR.
//
// Everything is laid out on a 1600x900 canvas and scaled to the window, so the layout is
// the approved mockup's at any size and any DPI; text is drawn at the device's own pixels
// (Theme.Draw). Pop-ups are drawn in the same window (Dialogs.cs).
using System;
using System.Collections.Generic;
using System.Drawing;
using System.Drawing.Drawing2D;
using System.Drawing.Text;
using System.Globalization;
using System.IO;
using System.Linq;
using System.Runtime.InteropServices;
using System.Threading;
using System.Windows.Forms;

namespace KmrpFontConverter
{
    internal sealed partial class MainForm : Form
    {
        private const float CanvasW = 1600, CanvasH = 900;

        private readonly List<Tile> tiles;
        private readonly Dictionary<string, Reference> references;
        private readonly Bitmap kmrpLogo, kotorLogo, gear;
        private readonly Dictionary<string, Bitmap> scaled = new Dictionary<string, Bitmap>();
        private string output;
        private Target target;

        private float scale = 1, offsetX, offsetY;
        private readonly List<Hit> hits = new List<Hit>();
        private string hover, pressed;
        private Dialog dialog;
        private Tile dropTarget;
        private volatile int progress = -1, progressTotal;
        private volatile bool stopRequested;
        private bool still, exitAfterConvert, closeWhenDone;
        private bool converted;     // the last Convert wrote what the tiles show; any change clears it

        /// <summary>The program's exit code: 1 when a conversion asked for by --convert --exit
        /// failed, else 0.</summary>
        public int ExitCode { get; private set; }

        private sealed class Hit
        {
            public RectangleF Rect;
            public string Id;
            public Action Click;
        }

        public MainForm(string[] args)
        {
            Text = "KMRP Font Converter";
            Places.LoadLook();
            BackColor = Theme.Window;
            DoubleBuffered = true;
            SetStyle(ControlStyles.AllPaintingInWmPaint | ControlStyles.OptimizedDoubleBuffer | ControlStyles.ResizeRedraw | ControlStyles.UserPaint, true);
            KeyPreview = true;
            AllowDrop = true;
            try { Icon = Icon.ExtractAssociatedIcon(Application.ExecutablePath); } catch (Exception) { }

            references = Reference.Parse(Program.ResourceText("references.txt"));
            tiles = Tiles.All().Select(d => new Tile { Def = d }).ToList();
            kmrpLogo = Program.ResourceImage("kmrp-logo.png");
            kotorLogo = Program.ResourceImage("kotor-logo.png");
            gear = Program.ResourceImage("settings.png");
            output = Places.SavedFolder() ?? Places.DefaultFolder();
            target = Places.LoadTarget(output);

            // 1600x900 at the screen's scale, as large as fits the work area.
            float dpi;
            using (var g = CreateGraphics()) dpi = g.DpiX / 96f;
            Rectangle work = Screen.PrimaryScreen.WorkingArea;
            float fit = Math.Min(dpi, Math.Min(work.Width * 0.94f / CanvasW, work.Height * 0.92f / CanvasH));
            ClientSize = new Size((int)(CanvasW * fit), (int)(CanvasH * fit));
            // Smallest: three quarters of the canvas at the screen's scale, where the smallest
            // text is still about 10 pixels at 100%; never more than the work area allows.
            float least = Math.Min(Math.Max(0.75f, dpi * 0.75f), Math.Min(work.Width * 0.94f / CanvasW, work.Height * 0.92f / CanvasH));
            Size frame = Size - ClientSize;
            MinimumSize = new Size((int)(CanvasW * least) + frame.Width, (int)(CanvasH * least) + frame.Height);
            StartPosition = FormStartPosition.CenterScreen;

            Preload(args);
        }

        // For screenshots, checks and testing/regression/Test-FontConverter.py:
        //   key=path[#face]       a tile's font (key as in Tiles.All, face by name)
        //   --pick key=path       open the picker for a tile (one argument, quoted)
        //   --out=folder          write to this folder (not remembered)
        //   --version=kmrp|original   make the fonts for this version (not remembered)
        //   --textsize=small|medium|large, --theme=dark|light   the look (not remembered)
        //   --convert             convert as soon as the window shows
        //   --exit                with --convert: close when it is done, exit code 0 or 1
        //   --size=WxH            the window's inside at this size
        //   --still               ignore the mouse (screenshots)
        //   --settings            open Settings
        private void Preload(string[] args)
        {
            foreach (string arg in args)
            {
                if (arg.StartsWith("--out=")) { output = arg.Substring(6); continue; }
                if (arg.StartsWith("--textsize="))
                {
                    int size = Array.IndexOf(Places.TextSizes, arg.Substring(11));
                    if (size >= 0) Theme.TextScale = Places.TextScales[size];
                    continue;
                }
                if (arg == "--theme=light" || arg == "--theme=dark") { Theme.Light = arg == "--theme=light"; BackColor = Theme.Window; continue; }
                if (arg == "--version=kmrp") { target = Target.Kmrp; continue; }
                if (arg == "--version=original") { target = Target.Original; continue; }
                if (arg.StartsWith("--size="))
                {
                    string[] wh = arg.Substring(7).Split('x');
                    int w, h;
                    if (wh.Length == 2 && int.TryParse(wh[0], out w) && int.TryParse(wh[1], out h) && w > 0 && h > 0)
                        ClientSize = new Size(w, h);
                    continue;
                }
                if (arg == "--convert") { Shown += (s, e) => Convert(); continue; }
                if (arg == "--exit") { exitAfterConvert = true; continue; }
                if (arg == "--still") { still = true; continue; }
                if (arg == "--settings") { dialog = Settings(); continue; }
                bool pick = arg.StartsWith("--pick ");
                string spec = pick ? arg.Substring(7) : arg;
                int eq = spec.IndexOf('=');
                if (eq < 0) continue;
                Tile tile = tiles.FirstOrDefault(t => t.Def.Key == spec.Substring(0, eq));
                if (tile == null) continue;
                string path = spec.Substring(eq + 1), face = null;
                int hash = path.IndexOf('#');
                if (hash >= 0) { face = path.Substring(hash + 1); path = path.Substring(0, hash); }
                // As if the player had chosen the file: the same checks and messages.
                if (pick) { Open(tile, path); continue; }
                List<FontFaceChoice> faces;
                try { faces = FontFiles.Faces(path); }
                catch (Exception ex) { Message("This font can't be read", ex.Message); continue; }
                var choice = faces.FirstOrDefault(f => face == null || f.Name == face) ?? faces[0];
                Apply(tile, choice, faces);
            }
        }

        protected override void OnHandleCreated(EventArgs e)
        {
            base.OnHandleCreated(e);
            TitleBar();
        }

        // A dark title bar in dark mode: attribute 20 on Windows 10 2004 and later, 19 on
        // 1809 to 1909; ignored elsewhere (Wine).
        private void TitleBar()
        {
            try
            {
                int dark = Theme.Light ? 0 : 1;
                if (DwmSetWindowAttribute(Handle, 20, ref dark, 4) != 0) DwmSetWindowAttribute(Handle, 19, ref dark, 4);
            }
            catch (Exception) { }
        }

        /// <summary>A text size or theme picked in Settings: kept, and shown at once.</summary>
        private void SetLook(int size, bool light)
        {
            Theme.TextScale = Places.TextScales[size];
            Theme.Light = light;
            Places.Set("textsize", Places.TextSizes[size]);
            Places.Set("theme", light ? "light" : "dark");
            BackColor = Theme.Window;
            TitleBar();
            Invalidate();
        }

        [DllImport("dwmapi.dll")]
        private static extern int DwmSetWindowAttribute(IntPtr hwnd, int attribute, ref int value, int size);

        // ---- painting ----

        protected override void OnPaint(PaintEventArgs e)
        {
            Graphics g = e.Graphics;
            scale = Math.Min(ClientSize.Width / CanvasW, ClientSize.Height / CanvasH);
            offsetX = (float)Math.Round((ClientSize.Width - CanvasW * scale) / 2);
            offsetY = (float)Math.Round((ClientSize.Height - CanvasH * scale) / 2);
            g.Clear(Theme.Window);
            g.SmoothingMode = SmoothingMode.AntiAlias;
            // Grid-fitted: the fonts' hints put stems on whole pixels (plain AntiAlias ignored
            // them and left the text soft).
            g.TextRenderingHint = TextRenderingHint.AntiAliasGridFit;
            g.PixelOffsetMode = PixelOffsetMode.HighQuality;
            g.InterpolationMode = InterpolationMode.HighQualityBicubic;
            g.TranslateTransform(offsetX, offsetY);
            g.ScaleTransform(scale, scale);
            hits.Clear();

            PaintHeader(g);
            PaintGroups(g);
            PaintActions(g);
            if (dialog != null)
            {
                hits.Clear();
                using (var dim = new SolidBrush(Theme.Dim)) g.FillRectangle(dim, -offsetX / scale, -offsetY / scale, ClientSize.Width / scale, ClientSize.Height / scale);
                dialog.Paint(this, g);
            }
        }

        private void AddHit(RectangleF rect, string id, Action click)
        {
            hits.Add(new Hit { Rect = rect, Id = id, Click = click });
        }

        private bool Hover(string id) { return hover == id; }

        private void PaintHeader(Graphics g)
        {
            using (var path = new GraphicsPath())
            {
                // From the window's own top and sides, where the canvas does not fill it.
                float left = -offsetX / scale, edge = -offsetY / scale, width = ClientSize.Width / scale;
                path.AddEllipse(288 - 900, edge - 260, 1800, 520);
                using (var glow = new PathGradientBrush(path) { CenterColor = Theme.HeaderGlow, SurroundColors = new[] { Theme.Window }, CenterPoint = new PointF(288, edge) })
                    g.FillRectangle(glow, left, edge, width, 218 - edge);
                using (var fade = new LinearGradientBrush(new RectangleF(left, 86, width, 133), Color.FromArgb(0, Theme.Window), Theme.Window, 90f))
                    g.FillRectangle(fade, left, 87, width, 131);
            }

            // The logo: KMRP's, or the same lockup saying KOTOR when the fonts are made for the
            // original game (the maintainer, 2026-11-08). Both images are made at one scale
            // (tools/build_brand_image.py, 714x535 made 400x300), so their letters match.
            Bitmap logo = target == Target.Original && kotorLogo != null ? kotorLogo : kmrpLogo;
            float logoW = 0, logoTop = 14, logoH = 116;
            if (logo != null)
            {
                float drawnH = logoH * logo.Height / 300f;
                logoW = drawnH * logo.Width / logo.Height;
                DrawSharp(g, logo, logo == kotorLogo ? "logo:kotor" : "logo:kmrp", new RectangleF(56, logoTop, logoW, drawnH));
            }
            // The title's capitals centred on the logo's middle.
            Font title = Theme.Display(42);
            float cap = 0.70f * title.Size, middle = logoTop + logoH / 2 + 1;
            float ascent = title.Size * title.FontFamily.GetCellAscent(title.Style) / title.FontFamily.GetEmHeight(title.Style);
            float top = middle + cap / 2 - ascent, titleX = 56 + logoW + 26;
            if (!Theme.Light) Theme.DrawAt(g, "Font Converter", title, Color.FromArgb(120, 0, 0, 0), titleX + 1, top + 2);
            Theme.DrawAt(g, "Font Converter", title, Theme.Text, titleX, top);

            // Settings, top right: the installer's square gear button (card fill, quiet edge,
            // the accent on hover), the size of a tile's foot button and then some.
            var settings = new RectangleF(1488, middle - 26, 52, 52);
            bool on = Hover("settings");
            Theme.Box(g, settings, 6, on ? Theme.ButtonHover : Theme.Button, on ? Theme.Accent : Theme.Edge);
            DrawMask(g, gear, "gear", new RectangleF(settings.X + 9, settings.Y + 9, 34, 34), on ? Theme.Accent : Theme.Text);
            AddHit(settings, "settings", () => { if (progress < 0) { dialog = Settings(); Invalidate(); } });

            // The introduction, between the title and the button, centred on the logo's middle.
            // Shorter since 2026-11-08 (audit T1): "Anything you don't pick stays as it is" was
            // no longer true once KMRP's Modded Font shows its HD Font for the names not picked.
            float introX = Math.Max(800, titleX + Theme.Width(g, "Font Converter", title) + 48);
            var intro = new RectangleF(introX, 0, settings.X - 28 - introX, 0);
            string[] runs = { "Pick a font for any text you want to change. ", "*Every size the game needs*", " is made from it." };
            float introH = Rich(g, intro, 16, Theme.Muted, Theme.Text, false, runs);
            intro.Y = Math.Max(8, middle - introH / 2 - 2);
            Rich(g, intro, 16, Theme.Muted, Theme.Text, true, runs);
        }

        // Wrapped text with *bold* runs; its height. Measured only, when `draw` is false.
        private static float Rich(Graphics g, RectangleF r, float px, Color colour, Color strong, bool draw, params string[] runs)
        {
            float x = r.X, y = r.Y, line = Theme.T(px) * 1.55f;
            foreach (string run in runs)
            {
                bool bold = run.StartsWith("*");
                Font font = Theme.Body(px, bold);
                string text = run.Trim('*');
                foreach (string word in System.Text.RegularExpressions.Regex.Split(text, @"(?<= )"))
                {
                    if (word.Length == 0) continue;
                    float w = Theme.Width(g, word, font);
                    if (x + w > r.Right + 0.5f && x > r.X) { x = r.X; y += line; }
                    if (draw) Theme.DrawAt(g, word, font, bold ? strong : colour, x, y);
                    // The word, then its space measured on its own: GDI+ leaves trailing
                    // spaces out of a width, and a no-break space too when the font draws it
                    // with the space's glyph (as KMRP's fonts do).
                    x += Theme.Width(g, word.TrimEnd(' '), font);
                    if (word.EndsWith(" ")) x += Theme.Width(g, "a a", font) - Theme.Width(g, "aa", font);
                }
            }
            return y - r.Y + line;
        }

        private static readonly string[][] Rows = { new[] { "Menus" }, new[] { "Reading", "HUD", "Credits" } };

        // Each group's colour, darker in light mode so it reads on the pale panels.
        private static KeyValuePair<Color, string> GroupInfo(string name)
        {
            bool light = Theme.Light;
            switch (name)
            {
                case "Menus": return new KeyValuePair<Color, string>(Theme.Gold, "Every screen you click through");
                case "Reading": return new KeyValuePair<Color, string>(light ? Color.FromArgb(21, 101, 192) : Color.FromArgb(96, 178, 255), "Conversations and the journal");
                case "HUD": return new KeyValuePair<Color, string>(Theme.Done, "On screen while you play");
                default: return new KeyValuePair<Color, string>(light ? Color.FromArgb(118, 62, 184) : Color.FromArgb(197, 140, 255), "The end credits");
            }
        }

        private void PaintGroups(Graphics g)
        {
            float y = 146;
            foreach (string[] row in Rows)
            {
                int count = row.Sum(n => tiles.Count(t => t.Def.Group == n));
                float available = 1480 - 14 * (row.Length - 1), x = 60;
                foreach (string name in row)
                {
                    var members = tiles.Where(t => t.Def.Group == name).ToList();
                    float w = available * members.Count / count;
                    var r = new RectangleF(x, y, w, 291);
                    Theme.Box(g, r, 12, Theme.Group, Theme.GroupEdge);
                    var info = GroupInfo(name);
                    using (var dot = new SolidBrush(info.Key)) g.FillEllipse(dot, x + 16, y + 18, 8, 8);
                    float tx = x + 34;
                    Font head = Theme.Display(15);
                    foreach (char c in name.ToUpperInvariant())
                    {
                        Theme.DrawAt(g, c.ToString(), head, info.Key, tx, y + 13);
                        tx += Theme.Width(g, c.ToString(), head) + 3;
                    }
                    Theme.Draw(g, info.Value, Theme.Body(14), Theme.Soft, new RectangleF(tx + 9, y + 22 - Theme.T(11), r.Right - tx - 20, Theme.T(22)));
                    float tw = (w - 24 - 12 * (members.Count - 1)) / members.Count;
                    for (int i = 0; i < members.Count; i++)
                        PaintTile(g, members[i], new RectangleF(x + 12 + i * (tw + 12), y + 43, tw, 236));
                    x += w + 14;
                }
                y += 291 + 14;
            }
        }

        private void PaintTile(Graphics g, Tile tile, RectangleF r)
        {
            tile.Bounds = r;
            bool mine = tile.Face != null;
            string id = "tile:" + tile.Def.Key;
            bool lit = dropTarget == tile;
            // A picked font lights its tile orange; once converted, green.
            Color state = tile.Converted ? Theme.Done : Theme.Picked;
            Color fill = lit ? Theme.TileHover : Theme.Tile;
            Theme.Box(g, r, 8, fill, lit ? Theme.Accent : mine ? state : Theme.TileEdge, lit || mine ? 2 : 1);
            if (mine && !lit)
                using (var path = Theme.Round(r, 8))
                using (var glow = new LinearGradientBrush(new RectangleF(r.X, r.Y - 1, r.Width, r.Height * 0.6f + 2), Color.FromArgb(48, state), Color.FromArgb(0, state), 90f))
                {
                    g.SetClip(path);
                    g.FillRectangle(glow, r.X, r.Y, r.Width, r.Height * 0.6f);
                    g.ResetClip();
                }

            Theme.Draw(g, tile.Def.Name, Theme.Display(19), Theme.Text, new RectangleF(r.X + 14, r.Y + 10, r.Width - 28, Theme.T(26)));
            // What the tile is for, on two lines where a narrow tile needs them at a large
            // text size.
            Font use = Theme.Body(14);
            float useY = r.Y + 10 + Theme.T(26), useLine = Theme.T(19);
            int useLines = Theme.Width(g, tile.Def.Use, use) <= r.Width - 28 ? 1 : 2;
            Theme.DrawWrapped(g, tile.Def.Use, use, Theme.Muted, new RectangleF(r.X + 14, useY, r.Width - 28, useLine * useLines + 2));

            float footH = Math.Max(34, Theme.T(30)), boxY = useY + useLine * useLines + 8;
            var box = new RectangleF(r.X + 14, boxY, r.Width - 28, r.Bottom - 12 - footH - 10 - boxY);
            Theme.Box(g, box, 5, Theme.Example, Theme.ExampleEdge);
            PaintExample(g, tile, box);

            var foot = new RectangleF(r.X + 14, r.Bottom - 12 - footH, r.Width - 28, footH);
            if (!mine)
            {
                bool on = Hover(id + ":choose");
                using (var path = Theme.Round(foot, 5))
                {
                    if (on) using (var b = new SolidBrush(Theme.ButtonHover)) g.FillPath(b, path);
                    using (var pen = new Pen(on ? Theme.Accent : Theme.Dashed) { DashPattern = new[] { 3f, 2f } }) g.DrawPath(pen, path);
                }
                Theme.Draw(g, "Choose a font\u2026", Theme.Display(15), on ? Theme.Text : Theme.ChooseInk, foot, Theme.Centre);
                AddHit(foot, id + ":choose", () => Choose(tile));
            }
            else
            {
                var clear = new RectangleF(foot.Right - Theme.T(32), foot.Y, Theme.T(32), foot.Height);
                var chip = new RectangleF(foot.X, foot.Y, foot.Width, foot.Height);
                bool on = Hover(id + ":chip");
                Theme.Box(g, chip, 5, Color.FromArgb(on ? 40 : 22, state), Color.FromArgb(150, state));
                // The font's name; a warning beside it only when the font lacks letters the
                // game's text uses (those are left blank).
                Font name = Theme.Display(15), small = Theme.Body(13);
                string warning = tile.Missing.Count == 0 ? null : "\u26A0 " + tile.Missing.Count + " missing";
                float warnW = warning == null ? 0 : Theme.Width(g, warning, small) + 8;
                float nameW = Math.Min(Theme.Width(g, tile.Face.Name, name) + 2, chip.Width - 20 - clear.Width - warnW);
                Theme.Draw(g, tile.Face.Name, name, state, new RectangleF(chip.X + 10, chip.Y, Math.Max(20, nameW), chip.Height));
                if (warning != null)
                    Theme.Draw(g, warning, small, Theme.Warning, new RectangleF(chip.X + 10 + Math.Max(20, nameW) + 8, chip.Y + 1, warnW, chip.Height));
                bool x = Hover(id + ":clear");
                Theme.Draw(g, "\u2715", Theme.Body(15), x ? Theme.Text : Theme.Faint, clear, Theme.Centre);
                AddHit(chip, id + ":chip", () => Choose(tile));
                AddHit(clear, id + ":clear", () => Clear(tile));
            }
        }

        // The tile's example lines, each piece in the HD Original font or the picked one,
        // all of a tile's lines shrunk together when the widest would not fit.
        private void PaintExample(Graphics g, Tile tile, RectangleF box)
        {
            bool mine = tile.Face != null;
            const float gap = 18, lineGap = 9;
            float room = box.Width - 24;
            var widths = new List<float>();
            foreach (var line in tile.Def.Lines)
                widths.Add(line.Sum(p => PieceWidth(tile, p)) + gap * (line.Count - 1));
            // Shrunk to the box's width, and to its height where a large text size left it short.
            float tall = tile.Def.Lines.Sum(l => l.Max(p => p.Height)), gaps = lineGap * (tile.Def.Lines.Count - 1);
            float shrink = Math.Min(1, Math.Min(room / Math.Max(1, widths.Max()), (box.Height - 16 - gaps) / Math.Max(1, tall)));
            float total = tall * shrink + gaps;
            float y = box.Y + (box.Height - total) / 2;
            for (int n = 0; n < tile.Def.Lines.Count; n++)
            {
                var line = tile.Def.Lines[n];
                float lineH = line.Max(p => p.Height) * shrink;
                float x = tile.Def.Centre ? box.X + (box.Width - widths[n] * shrink) / 2 : box.X + 12;
                foreach (var piece in line)
                {
                    float h = piece.Height * shrink;
                    float w = PieceWidth(tile, piece) * shrink;
                    if (mine) DrawPreview(g, tile, piece, new RectangleF(x, y + (lineH - h) / 2, w, h));
                    else DrawSharp(g, piece.Hd, piece.Name, new RectangleF(x, y + (lineH - h) / 2, w, h));
                    x += w + gap * shrink;
                }
                y += lineH + lineGap;
            }
        }

        // A piece's width at its own height, on the 1600x900 canvas.
        private float PieceWidth(Tile tile, Piece piece)
        {
            if (tile.Face == null) return piece.Hd.Width * piece.Height / piece.Hd.Height;
            Bitmap measure = Preview(tile, piece, piece.Height * 2, true);
            return measure == null ? piece.Height : (measure.Width - 2 * (Pad(measure) - 1)) / 2f;
        }

        // A piece in the picked font, `height` device pixels high. One bitmap a piece (and one
        // to measure with, "measure:" keys), replaced when the size changes, so resizing
        // never piles them up. Null when the face cannot draw it (audit: a failure here was
        // an exception in the middle of painting, on every repaint).
        private Bitmap Preview(Tile tile, Piece piece, float height, bool measure = false)
        {
            int h = Math.Max(4, (int)Math.Round(height));
            string key = (measure ? "measure:" : "") + piece.Name;
            Bitmap b;
            if (tile.Cache.TryGetValue(key, out b) && b != null && Box(b) == h) return b;
            if (b != null) b.Dispose();
            // The box plus a blank pixel round it (and room for the HUD font's black ring:
            // one original pixel, a sixth of its 6x atlas's 96 pixel glyph box per 16).
            float ring = Atlas.Outlined.ContainsKey(piece.Resref) ? h * Atlas.Outlined[piece.Resref] / 96f : 0;
            try { b = Atlas.Line(tile.Face, piece.Text, h, piece.Colour, ring); }
            catch (Exception) { b = null; }
            tile.Cache[key] = b;
            return b;
        }

        private static int Box(Bitmap b) { return ((int[])b.Tag)[0]; }
        private static int Pad(Bitmap b) { return ((int[])b.Tag)[1]; }

        private void DrawPreview(Graphics g, Tile tile, Piece piece, RectangleF r)
        {
            Bitmap b;
            // While the window is dragged to a new size the last bitmap is stretched; the
            // sharp one is drawn when the drag ends.
            if (resizing && tile.Cache.TryGetValue(piece.Name, out b) && b != null)
            {
                float f = r.Height * scale / Math.Max(1, Box(b));
                float m = Pad(b) * f / scale;
                DrawStretched(g, b, r.X - m, r.Y - m, b.Width * f, b.Height * f);
                return;
            }
            b = Preview(tile, piece, r.Height * scale);
            if (b != null) DrawDevice(g, b, r.X - (Pad(b) - 1) / scale, r.Y - (Pad(b) - 1) / scale);
        }

        /// <summary>An image drawn at the device's own pixels: scaled once into a cache, so
        /// painting never resamples and stays fast.</summary>
        private void DrawSharp(Graphics g, Bitmap source, string name, RectangleF r, Color? tint = null)
        {
            if (source == null) return;
            int w = Math.Max(1, (int)Math.Round(r.Width * scale)), h = Math.Max(1, (int)Math.Round(r.Height * scale));
            Bitmap b;
            // One bitmap an image, made again when its size changes; stretched while resizing.
            if (scaled.TryGetValue(name, out b) && (b.Width != w || b.Height != h))
            {
                if (resizing) { DrawStretched(g, b, r.X, r.Y, w, h); return; }
                b.Dispose();
                b = null;
            }
            if (b == null)
            {
                b = new Bitmap(w, h, System.Drawing.Imaging.PixelFormat.Format32bppPArgb);
                using (var s = Graphics.FromImage(b))
                {
                    s.InterpolationMode = InterpolationMode.HighQualityBicubic;
                    s.PixelOffsetMode = PixelOffsetMode.HighQuality;
                    s.CompositingQuality = CompositingQuality.HighQuality;
                    using (var attributes = new System.Drawing.Imaging.ImageAttributes())
                    {
                        attributes.SetWrapMode(WrapMode.TileFlipXY);
                        s.DrawImage(source, new Rectangle(0, 0, w, h), 0, 0, source.Width, source.Height, GraphicsUnit.Pixel, attributes);
                    }
                }
                scaled[name] = b;
            }
            if (tint == null) { DrawDevice(g, b, r.X, r.Y); return; }
            Color c = tint.Value;
            var state = g.Save();
            g.ResetTransform();
            using (var attributes = new System.Drawing.Imaging.ImageAttributes())
            {
                attributes.SetColorMatrix(new System.Drawing.Imaging.ColorMatrix(new float[][] {
                    new float[] { 0, 0, 0, 0, 0 },
                    new float[] { 0, 0, 0, 0, 0 },
                    new float[] { 0, 0, 0, 0, 0 },
                    new float[] { 0, 0, 0, c.A / 255f, 0 },
                    new float[] { c.R / 255f, c.G / 255f, c.B / 255f, 0, 1 },
                }));
                float x = (float)Math.Round(offsetX + r.X * scale), y = (float)Math.Round(offsetY + r.Y * scale);
                g.DrawImage(b, new Rectangle((int)x, (int)y, b.Width, b.Height), 0, 0, b.Width, b.Height, GraphicsUnit.Pixel, attributes);
            }
            g.Restore(state);
        }

        /// <summary>Line art drawn in one colour from its alpha (the installer's DrawIconMask):
        /// the gear is black on transparency, so the colour replaces the source's rather than
        /// multiplying it. Cropped to its ink once, scaled once a size, drawn 1:1.</summary>
        private void DrawMask(Graphics g, Bitmap source, string name, RectangleF r, Color colour)
        {
            if (source == null) return;
            Bitmap ink;
            if (!scaled.TryGetValue(name + ":ink", out ink))
            {
                Rectangle bounds = Ink(source);
                ink = source.Clone(bounds, source.PixelFormat);
                scaled[name + ":ink"] = ink;
            }
            float fit = Math.Min(r.Width / ink.Width, r.Height / ink.Height);
            var box = new RectangleF(r.X + (r.Width - ink.Width * fit) / 2, r.Y + (r.Height - ink.Height * fit) / 2, ink.Width * fit, ink.Height * fit);
            DrawSharp(g, ink, name, box, colour);
        }

        // The opaque part of an image.
        private static Rectangle Ink(Bitmap b)
        {
            int left = b.Width, top = b.Height, right = -1, bottom = -1;
            for (int y = 0; y < b.Height; y += 2)
                for (int x = 0; x < b.Width; x += 2)
                    if (b.GetPixel(x, y).A > 8)
                    {
                        if (x < left) left = x;
                        if (x > right) right = x;
                        if (y < top) top = y;
                        if (y > bottom) bottom = y;
                    }
            if (right < 0) return new Rectangle(0, 0, b.Width, b.Height);
            left = Math.Max(0, left - 2); top = Math.Max(0, top - 2);
            return Rectangle.FromLTRB(left, top, Math.Min(b.Width, right + 3), Math.Min(b.Height, bottom + 3));
        }

        private void DrawStretched(Graphics g, Bitmap b, float x, float y, float w, float h)
        {
            var state = g.Save();
            g.ResetTransform();
            g.InterpolationMode = InterpolationMode.Bilinear;
            g.DrawImage(b, (float)Math.Round(offsetX + x * scale), (float)Math.Round(offsetY + y * scale), w, h);
            g.Restore(state);
        }

        private bool resizing;

        // Dragging an edge keeps the inside 16:9, the canvas's shape, so it is always filled:
        // the other side follows (the bottom for a side edge or corner, the right for the
        // top or bottom edge). Maximized on a screen of another shape, the canvas is centred.
        protected override void WndProc(ref Message m)
        {
            const int WM_SIZING = 0x0214;
            if (m.Msg == WM_SIZING && m.LParam != IntPtr.Zero)
            {
                var r = (WindowRect)Marshal.PtrToStructure(m.LParam, typeof(WindowRect));
                Size frame = Size - ClientSize;
                int edge = m.WParam.ToInt32();
                int w = r.Right - r.Left - frame.Width, h = r.Bottom - r.Top - frame.Height;
                if (edge == 3 || edge == 6)                     // top, bottom: the width follows
                    r.Right = r.Left + (int)Math.Round(h * CanvasW / CanvasH) + frame.Width;
                else
                {
                    int height = (int)Math.Round(w * CanvasH / CanvasW) + frame.Height;
                    if (edge == 4 || edge == 5) r.Top = r.Bottom - height;   // top corners
                    else r.Bottom = r.Top + height;
                }
                Marshal.StructureToPtr(r, m.LParam, false);
                m.Result = (IntPtr)1;
                return;
            }
            base.WndProc(ref m);
        }

        [StructLayout(LayoutKind.Sequential)]
        private struct WindowRect { public int Left, Top, Right, Bottom; }

        protected override void OnResizeBegin(EventArgs e) { base.OnResizeBegin(e); resizing = true; }
        protected override void OnResizeEnd(EventArgs e) { base.OnResizeEnd(e); resizing = false; Invalidate(); }

        private void DrawDevice(Graphics g, Bitmap b, float x, float y)
        {
            var state = g.Save();
            g.ResetTransform();
            g.InterpolationMode = InterpolationMode.NearestNeighbor;
            g.DrawImage(b, (float)Math.Round(offsetX + x * scale), (float)Math.Round(offsetY + y * scale), b.Width, b.Height);
            g.Restore(state);
        }

        // The bottom row: the count, the folder, Clear, the Version box and Convert.
        private const float ActionsY = 770, ActionsH = 68;

        private void PaintActions(Graphics g)
        {
            int chosen = tiles.Count(t => t.Face != null);
            bool busy = progress >= 0;
            Font count = Theme.Display(20);
            float x = 60, row = ActionsY, rowH = ActionsH, middle = row + rowH / 2;
            Theme.Draw(g, chosen.ToString(), count, Theme.Gold, new RectangleF(x, middle - 20, 60, 40));
            // The font's own word space after the number, as between "of" and "11" (until
            // 2026-11-08 a fixed 6, narrower: the maintainer saw "2 of 11" unevenly spaced).
            x += Theme.Width(g, chosen.ToString(), count) + Theme.Width(g, "a a", count) - Theme.Width(g, "aa", count);
            string ofAll = "of 11 fonts chosen";
            Theme.Draw(g, ofAll, count, Theme.Text, new RectangleF(x, middle - 20, Theme.Width(g, ofAll, count) + 4, 40));
            x += Theme.Width(g, ofAll, count) + 24;

            // Convert, the Version box and Clear from the right, each as wide as its longest
            // label needs; the folder takes what is left.
            Font big = Theme.Display(21), mid = Theme.Display(19), choice = Theme.Display(14);
            float convertW = Math.Max(220, Math.Max(Theme.Width(g, "Convert", big), Theme.T(17) + 10 + Theme.Width(g, "Converted", big)) + 64);
            float versionW = Math.Max(150, 44 + Math.Max(Theme.Width(g, "Original Game", choice), Theme.Width(g, "KMRP", choice)) + 16);
            float clearW = Math.Max(120, Theme.Width(g, "Clear", mid) + 50);
            var convert = new RectangleF(1540 - convertW, row, convertW, rowH);
            var version = new RectangleF(convert.X - 10 - versionW, row, versionW, rowH);
            var clear = new RectangleF(version.X - 14 - clearW, row, clearW, rowH);

            var path = new RectangleF(x, row, clear.X - 14 - x, rowH);
            Theme.Box(g, path, 5, Theme.Field, Theme.Edge);
            Font change = Theme.Display(16);
            float changeW = Math.Max(86, Theme.Width(g, "Change", change) + 28);
            var changeBox = new RectangleF(path.Right - 14 - changeW, path.Y + (rowH - 38) / 2, changeW, 38);
            float labelH = Theme.T(16), pathH = Theme.T(22), top = path.Y + (path.Height - labelH - pathH) / 2;
            Theme.Draw(g, "SAVE TO", Theme.Body(12.5f), Theme.Faint, new RectangleF(path.X + 16, top, 200, labelH));
            Theme.Draw(g, output, Theme.Mono(14), Theme.Muted, new RectangleF(path.X + 16, top + labelH, changeBox.X - path.X - 30, pathH), Theme.Path);
            bool on = !busy && Hover("output");
            Theme.Box(g, changeBox, 5, on ? Theme.ButtonHover : Theme.Button, on ? Theme.Accent : Theme.Edge);
            Theme.Draw(g, "Change", change, busy ? Theme.Faint : Theme.Text, changeBox, Theme.Centre);
            if (!busy) AddHit(changeBox, "output", PickFolder);

            // Clear: every tile back to the Original HD Font.
            bool canClear = chosen > 0 && !busy, onClear = canClear && Hover("clearall");
            // A grey button: set apart from the folder's field, quieter than Convert.
            Color clearTop, clearBottom;
            if (Theme.Light)
            {
                clearTop = canClear ? (onClear ? Color.FromArgb(214, 222, 231) : Color.FromArgb(201, 210, 220)) : Color.FromArgb(226, 232, 239);
                clearBottom = canClear ? (onClear ? Color.FromArgb(195, 204, 214) : Color.FromArgb(182, 192, 203)) : Color.FromArgb(218, 225, 233);
            }
            else
            {
                clearTop = canClear ? (onClear ? Color.FromArgb(104, 117, 134) : Color.FromArgb(86, 98, 114)) : Color.FromArgb(40, 48, 60);
                clearBottom = canClear ? (onClear ? Color.FromArgb(82, 94, 110) : Color.FromArgb(66, 77, 92)) : Color.FromArgb(32, 39, 50);
            }
            using (var p = Theme.Round(clear, 5))
            using (var b = new LinearGradientBrush(clear, clearTop, clearBottom, 90f))
            {
                g.FillPath(b, p);
                // An edge, so the button keeps its shape against the window in both modes.
                using (var pen = new Pen(Theme.Light ? Color.FromArgb(150, 165, 182) : Color.FromArgb(108, 122, 140))) g.DrawPath(pen, p);
            }
            Color offInk = Theme.Light ? Color.FromArgb(150, 162, 176) : Color.FromArgb(92, 104, 118);
            Theme.Draw(g, "Clear", mid, canClear ? Theme.Text : offInk, clear, Theme.Centre);
            if (canClear) AddHit(clear, "clearall", ClearAll);

            PaintVersion(g, version, busy);

            // Convert, or Converted while every picked font is written to this folder as it is.
            bool done = chosen > 0 && !busy && converted;
            bool ready = chosen > 0 && !busy && !done;
            using (var p = Theme.Round(convert, 5))
            {
                if (ready)
                {
                    Color hi = Hover("convert") ? Theme.AccentHover : Theme.AccentTop;
                    using (var b = new LinearGradientBrush(convert, hi, Theme.AccentBottom, 90f)) g.FillPath(b, p);
                }
                else if (done)
                    using (var b = new LinearGradientBrush(convert, Color.FromArgb(146, 234, 188), Color.FromArgb(96, 204, 148), 90f)) g.FillPath(b, p);
                else
                {
                    using (var b = new SolidBrush(Theme.Light ? Color.FromArgb(212, 228, 238) : Color.FromArgb(18, 40, 54))) g.FillPath(b, p);
                    using (var pen = new Pen(Theme.Light ? Color.FromArgb(150, 180, 200) : Color.FromArgb(44, 84, 106))) g.DrawPath(pen, p);
                }
                if (busy && progressTotal > 0)
                {
                    g.SetClip(p);
                    using (var b = new LinearGradientBrush(convert, Theme.AccentTop, Theme.AccentBottom, 90f))
                        g.FillRectangle(b, convert.X, convert.Y, convert.Width * progress / progressTotal, convert.Height);
                    g.ResetClip();
                }
            }
            Color waiting = Theme.Light ? Color.FromArgb(110, 140, 160) : Color.FromArgb(70, 110, 130);
            Color ink = ready || busy || done ? Theme.AccentInk : waiting;
            if (done)
            {
                // The tick drawn as two strokes: KMRP's fonts have only the game's letters, and
                // where Windows lends no other font's tick (Wine) a box stood in.
                float textW = Theme.Width(g, "Converted", big), tick = Theme.T(17);
                float tx = convert.X + (convert.Width - tick - 10 - textW) / 2, cy = convert.Y + convert.Height / 2;
                using (var pen = new Pen(ink, Math.Max(2.5f, Theme.T(2.6f))) { StartCap = LineCap.Round, EndCap = LineCap.Round, LineJoin = LineJoin.Round })
                    g.DrawLines(pen, new[] { new PointF(tx, cy), new PointF(tx + tick * 0.36f, cy + tick * 0.4f), new PointF(tx + tick, cy - tick * 0.48f) });
                Theme.Draw(g, "Converted", big, ink, new RectangleF(tx + tick + 10, convert.Y, textW + 6, convert.Height));
            }
            else
            {
                // The progress shrunk to fit Convert, which is as wide as "Converted" needs.
                string label = busy ? "Converting\u2026 " + progress + " of " + progressTotal : "Convert";
                Font font = big;
                float labelW = Theme.Width(g, label, big), room = convert.Width - 24;
                if (labelW > room) font = Theme.Display(21 * room / labelW);
                Theme.Draw(g, label, font, ink, convert, Theme.Centre);
            }
            if (ready) AddHit(convert, "convert", Convert);

            // The footer line, each piece after the last.
            Font foot = Theme.Body(15);
            float fy = 872 - Theme.T(12), fh = Theme.T(24), fx = 62;
            string versionText = "v" + Program.Version;
            Theme.Draw(g, versionText, foot, Theme.Faint, new RectangleF(fx, fy, Theme.Width(g, versionText, foot) + 4, fh));
            fx += Theme.Width(g, versionText, foot) + 12;
            Theme.Draw(g, "\u00B7", foot, Theme.Faint, new RectangleF(fx, fy, 12, fh));
            fx += Theme.Width(g, "\u00B7", foot) + 12;
            var open = new RectangleF(fx, fy, Theme.Width(g, "Open Output Folder", foot) + 4, fh);
            Theme.Draw(g, "Open Output Folder", foot, Hover("open") ? Theme.Text : Theme.Accent, open);
            AddHit(open, "open", OpenOutput);
            var credit = "Created by RaymanGT";
            float cw = Theme.Width(g, credit, foot);
            Theme.Draw(g, credit, foot, Theme.Faint, new RectangleF(1538 - cw, fy, cw + 4, fh));
        }

        // The Version box: "Version" in the window's yellow, then KMRP and Original Game with
        // KOTOR's round radio rings in its own blue (the maintainer's design, 2026-11-08:
        // "make it in the C style", "Version, not all caps", yellow).
        private void PaintVersion(Graphics g, RectangleF box, bool busy)
        {
            Theme.Box(g, box, 5, Theme.Field, Theme.Edge);
            Font label = Theme.Body(13), choice = Theme.Display(14);
            float labelH = Theme.T(16), rowsTop = box.Y + 6 + labelH, rowH = (box.Bottom - 5 - rowsTop) / 2;
            Theme.Draw(g, "Version", label, Theme.Gold, new RectangleF(box.X + 12, box.Y + 4, box.Width - 20, labelH));
            var options = new[] { new KeyValuePair<Target, string>(Target.Kmrp, "KMRP"), new KeyValuePair<Target, string>(Target.Original, "Original Game") };
            for (int i = 0; i < options.Length; i++)
            {
                Target t = options[i].Key;
                bool on = target == t;
                string id = "version:" + (t == Target.Kmrp ? "kmrp" : "original");
                bool lit = !busy && !on && Hover(id);
                var row = new RectangleF(box.X + 4, rowsTop + i * rowH, box.Width - 8, rowH);
                float ring = Math.Min(Theme.T(8), rowH / 2 - 2), cx = box.X + 12 + ring, cy = row.Y + rowH / 2;
                using (var pen = new Pen(lit ? Theme.Text : Theme.KotorBlue, Math.Max(1.6f, ring / 3.6f)))
                    g.DrawEllipse(pen, cx - ring, cy - ring, ring * 2, ring * 2);
                if (on)
                    using (var dot = new SolidBrush(Theme.KotorBlue))
                        g.FillEllipse(dot, cx - ring * 0.5f, cy - ring * 0.5f, ring, ring);
                Theme.Draw(g, options[i].Value, choice, on || lit ? Theme.Text : Theme.Faint,
                    new RectangleF(cx + ring + 9, row.Y, row.Right - cx - ring - 9, rowH));
                if (!busy && !on) AddHit(row, id, () => SetTarget(t));
            }
        }

        /// <summary>Another version chosen: kept, and what was converted for the other one is
        /// not what this one would write.</summary>
        private void SetTarget(Target value)
        {
            if (progress >= 0 || target == value) return;
            target = value;
            Places.SaveTarget(value);
            foreach (var tile in tiles) tile.Converted = false;
            converted = false;
            Invalidate();
        }

        private void OpenOutput()
        {
            try
            {
                Directory.CreateDirectory(output);
                Program.Open(output);
            }
            catch (Exception ex) { Message("The folder can't be opened", ex.Message); }
        }

        // ---- input ----

        private PointF Canvas(Point p)
        {
            return new PointF((p.X - offsetX) / scale, (p.Y - offsetY) / scale);
        }

        private Hit HitAt(Point p)
        {
            PointF c = Canvas(p);
            for (int i = hits.Count - 1; i >= 0; i--) if (hits[i].Rect.Contains(c)) return hits[i];
            return null;
        }

        protected override void OnMouseMove(MouseEventArgs e)
        {
            base.OnMouseMove(e);
            if (still) return;
            Hit hit = HitAt(e.Location);
            string id = hit == null ? null : hit.Id;
            Cursor = hit != null ? Cursors.Hand : Cursors.Default;
            if (id != hover) { hover = id; Invalidate(); }
        }

        protected override void OnMouseLeave(EventArgs e)
        {
            base.OnMouseLeave(e);
            if (hover != null) { hover = null; Invalidate(); }
        }

        protected override void OnMouseDown(MouseEventArgs e)
        {
            base.OnMouseDown(e);
            if (e.Button != MouseButtons.Left) return;
            Hit hit = HitAt(e.Location);
            pressed = hit == null ? null : hit.Id;
            if (dialog != null && e.Clicks == 2 && hit != null) dialog.DoubleClick(this, hit.Id);
        }

        protected override void OnMouseUp(MouseEventArgs e)
        {
            base.OnMouseUp(e);
            if (e.Button != MouseButtons.Left) return;
            Hit hit = HitAt(e.Location);
            if (hit != null && hit.Id == pressed && hit.Click != null) hit.Click();
            pressed = null;
        }

        protected override void OnMouseWheel(MouseEventArgs e)
        {
            base.OnMouseWheel(e);
            if (dialog != null) { dialog.Wheel(this, e.Delta); Invalidate(); }
        }

        protected override bool ProcessCmdKey(ref Message msg, Keys keyData)
        {
            if (dialog != null && dialog.Key(this, keyData)) { Invalidate(); return true; }
            return base.ProcessCmdKey(ref msg, keyData);
        }

        protected override void OnDragOver(DragEventArgs e)
        {
            base.OnDragOver(e);
            Tile over = null;
            if (dialog == null && progress < 0 && e.Data.GetDataPresent(DataFormats.FileDrop))
            {
                PointF c = Canvas(PointToClient(new Point(e.X, e.Y)));
                over = tiles.FirstOrDefault(t => t.Bounds.Contains(c));
            }
            e.Effect = over != null ? DragDropEffects.Copy : DragDropEffects.None;
            if (over != dropTarget) { dropTarget = over; Invalidate(); }
        }

        protected override void OnDragLeave(EventArgs e)
        {
            base.OnDragLeave(e);
            if (dropTarget != null) { dropTarget = null; Invalidate(); }
        }

        protected override void OnDragDrop(DragEventArgs e)
        {
            base.OnDragDrop(e);
            Tile target = dropTarget;
            dropTarget = null;
            var files = e.Data.GetData(DataFormats.FileDrop) as string[];
            if (target != null && files != null && files.Length > 0) BeginInvoke(new Action(() => Open(target, files[0])));
            Invalidate();
        }

        // Closing while fonts are being made: the conversion is stopped, nothing it made is
        // put in the folder, and the window closes when it has stopped (audit F3: the window
        // closed at once and the worker's last call then failed, or a cut file stayed).
        protected override void OnFormClosing(FormClosingEventArgs e)
        {
            if (progress >= 0)
            {
                stopRequested = true;
                closeWhenDone = true;
                e.Cancel = true;
                return;
            }
            base.OnFormClosing(e);
        }

        // ---- what the tiles do ----

        private void Choose(Tile tile)
        {
            if (progress >= 0) return;
            if (tile.FileFaces != null && tile.FileFaces.Count > 1)
            {
                dialog = Picker(tile, tile.FileFaces, true);
                Invalidate();
                return;
            }
            Browse(tile);
        }

        private void Browse(Tile tile)
        {
            using (var dialogBox = new OpenFileDialog
            {
                Title = "Choose a font for " + tile.Def.Name,
                Filter = "Fonts (*.ttf, *.otf, *.ttc, *.otc, *.woff, *.woff2)|*.ttf;*.otf;*.ttc;*.otc;*.woff;*.woff2|All files (*.*)|*.*",
            })
            {
                string folder = Places.LastFontFolder;
                if (folder != null) dialogBox.InitialDirectory = folder;
                if (dialogBox.ShowDialog(this) != DialogResult.OK) return;
                Places.LastFontFolder = Path.GetDirectoryName(dialogBox.FileName);
                Open(tile, dialogBox.FileName);
            }
        }

        private void Open(Tile tile, string path)
        {
            if (!FontFiles.Accepted(path))
            {
                Message("This font type can't be used", "Pick a TTF, OTF, TTC, OTC, WOFF or WOFF2 font file. " + Path.GetFileName(path) + " is not one.");
                return;
            }
            List<FontFaceChoice> faces;
            try
            {
                Cursor = Cursors.WaitCursor;
                faces = FontFiles.Faces(path);
            }
            catch (Exception ex)
            {
                Message("This font can't be read", ReadError(ex));
                return;
            }
            finally { Cursor = Cursors.Default; }
            if (faces.Count == 1) Apply(tile, faces[0], faces);
            else { dialog = Picker(tile, faces, false); Invalidate(); }
        }

        // The converter's own messages say what is wrong; anything else gets a plain sentence
        // and the system's words after it.
        private static string ReadError(Exception ex)
        {
            if (ex is InvalidDataException) return ex.Message;
            if (ex is FileNotFoundException || ex is DirectoryNotFoundException) return "The file is not there any more.";
            if (ex is UnauthorizedAccessException) return "Windows did not allow the file to be read.";
            return "The file could not be read as a font. (" + ex.Message + ")";
        }

        private void Apply(Tile tile, FontFaceChoice choice, List<FontFaceChoice> faces)
        {
            LoadedFace face;
            try { face = FontFiles.Load(choice); }
            catch (Exception ex) { Message("This font can't be read", ReadError(ex)); return; }
            // A font without the Latin letters would make the game's text invisible (audit F5).
            string missing;
            try { missing = Atlas.MissingBasicLatin(face); }
            catch (Exception ex) { face.Dispose(); Message("This font can't be read", ReadError(ex)); return; }
            if (missing.Length > 0)
            {
                face.Dispose();
                Message("This font can't be used", missing.Length >= 52
                    ? "It has no letters A to Z, so the game's text would be invisible. Pick a font made for Latin letters."
                    : "It has no " + string.Join(", ", missing.Select(c => c.ToString()).ToArray()) + ", so those letters would be invisible in the game. Pick a font with every letter from A to Z.");
                return;
            }
            Release(tile);
            tile.Choice = choice;
            tile.Face = face;
            tile.FileFaces = faces;
            try { tile.Missing = Atlas.Missing(face); }
            catch (Exception) { tile.Missing = new List<char>(); }
            tile.Converted = false;
            converted = false;
            Invalidate();
        }

        private void Release(Tile tile)
        {
            tile.Forget();
            if (tile.Face != null) tile.Face.Dispose();
            tile.Face = null;
            tile.Choice = null;
            tile.FileFaces = null;
            tile.Missing = null;
            tile.Converted = false;
        }

        private void Clear(Tile tile)
        {
            if (progress >= 0) return;
            Release(tile);
            converted = false;
            Invalidate();
        }

        private void ClearAll()
        {
            if (progress >= 0) return;
            foreach (var tile in tiles) Release(tile);
            converted = false;
            Invalidate();
        }

        private void PickFolder()
        {
            if (progress >= 0) return;
            using (var browse = new FolderBrowserDialog { Description = "Where the game fonts go. The game's Override folder is where the game reads them.", SelectedPath = output, ShowNewFolderButton = true })
            {
                if (browse.ShowDialog(this) != DialogResult.OK) return;
                if (!string.Equals(output, browse.SelectedPath, StringComparison.OrdinalIgnoreCase))
                {
                    foreach (var tile in tiles) tile.Converted = false;   // nothing written there yet
                    converted = false;
                }
                output = browse.SelectedPath;
                Places.Save(output);
                Invalidate();
            }
        }

        private void Convert()
        {
            if (progress >= 0) return;
            var chosen = tiles.Where(t => t.Face != null).ToList();
            if (chosen.Count == 0)
            {
                if (exitAfterConvert) { ExitCode = 1; Close(); }
                return;
            }
            int existing = Conversion.Existing(output, chosen.SelectMany(t => t.Def.Resrefs));
            if (existing > 0 && !exitAfterConvert)
            {
                dialog = new Dialog
                {
                    Title = "Replace fonts already there?",
                    Body = "The folder already has " + existing + " font file" + (existing == 1 ? "" : "s") + " with the names these fonts use. " +
                           "Converting replaces the .tga and .txi files. A .tpc file is kept, renamed to end in " + Conversion.TpcBackup + ".",
                    Buttons = { new DialogButton("Cancel", false, () => CloseDialog(null)), new DialogButton("Replace", true, () => { CloseDialog(null); Run(chosen); }) },
                };
                Invalidate();
                return;
            }
            Run(chosen);
        }

        // On the window's thread, unless the window is gone.
        private void OnWindow(Action action)
        {
            try { if (IsHandleCreated && !IsDisposed) BeginInvoke(action); }
            catch (InvalidOperationException) { }
        }

        private void Run(List<Tile> chosen)
        {
            var job = new Conversion
            {
                Folder = output,
                Target = target,
                References = references,
                Stop = () => stopRequested,
            };
            foreach (var t in chosen)
                foreach (string resref in t.Def.Resrefs) job.Work.Add(new KeyValuePair<string, LoadedFace>(resref, t.Face));
            job.Progress = n => { progress = n; OnWindow(Invalidate); };
            progressTotal = job.Work.Count;
            progress = 0;
            stopRequested = false;
            Invalidate();
            var thread = new Thread(() =>
            {
                Exception failed = null;
                try { job.Run(); }
                catch (Exception ex) { failed = ex; }
                OnWindow(() =>
                {
                    progress = -1;
                    if (closeWhenDone) { Close(); return; }
                    if (failed != null)
                    {
                        ExitCode = 1;
                        if (!(failed is OperationCanceledException))
                            Message("The fonts could not be written", failed.Message + (job.Moving
                                ? " Some fonts may already be in the folder: convert again to finish."
                                : failed.Message.Contains("Nothing was changed") ? "" : " Nothing in the folder was changed."));
                    }
                    else
                    {
                        // Done: Convert turns green ("Converted"), no pop-up (the maintainer, 2026-11-08).
                        foreach (var t in chosen) t.Converted = true;
                        converted = true;
                    }
                    Invalidate();
                    if (exitAfterConvert) Close();
                });
            }) { IsBackground = true };
            thread.Start();
        }

        private void Message(string title, string body)
        {
            dialog = new Dialog { Title = title, Body = body, Buttons = { new DialogButton("OK", true, () => CloseDialog(null)) } };
            Invalidate();
        }

        private void CloseDialog(Dialog next)
        {
            if (dialog != null) dialog.Closed();
            dialog = next;
            Invalidate();
        }
    }
}
