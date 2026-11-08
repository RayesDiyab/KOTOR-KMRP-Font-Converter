// KMRP Font Converter: the pop-ups, drawn in the window itself.
//
// Four kinds are left (2026-11-08, the maintainer: no pop-up at the start, "keep two", done
// as a green button): a message (a font that can't be read or used, fonts that could not be
// written), the question before replacing fonts already in the folder, Settings, and the
// picker for a collection's faces or a variable font's styles. The pop-up at every start
// that said the fonts need KMRP, with its Deadly Stream and GitHub buttons, and the Done
// pop-up after converting are gone: the program opens on its window, and Convert turns
// green when it is done.
using System;
using System.Collections.Generic;
using System.Drawing;
using System.Drawing.Drawing2D;
using System.Linq;
using System.Windows.Forms;

namespace KmrpFontConverter
{
    internal sealed partial class MainForm
    {
        private sealed class DialogButton
        {
            public string Label;
            public bool Primary;
            public Action Click;
            public DialogButton(string label, bool primary, Action click) { Label = label; Primary = primary; Click = click; }
        }

        private class Dialog
        {
            public string Title, Body;
            public float Width = 640;
            public List<DialogButton> Buttons = new List<DialogButton>();

            public virtual void Paint(MainForm form, Graphics g)
            {
                float bodyH = Lines(g, Body, Width - 56, Theme.Body(16)) * Theme.T(16) * 1.5f;
                float h = 26 + Theme.T(34) + 10 + bodyH + 24 + 48 + 24;
                var r = new RectangleF((CanvasW - Width) / 2, (CanvasH - h) / 2, Width, h);
                Frame(g, r);
                float y = r.Y + 26;
                Theme.Draw(g, Title, Theme.Display(26), Theme.Text, new RectangleF(r.X + 28, y, r.Width - 56, Theme.T(34)));
                y += Theme.T(34) + 10;
                Theme.DrawWrapped(g, Body, Theme.Body(16), Theme.Muted, new RectangleF(r.X + 28, y, r.Width - 56, bodyH + 4));
                y += bodyH + 24;
                PaintButtons(form, g, Buttons, r, y);
            }

            protected static int Lines(Graphics g, string text, float width, Font font)
            {
                if (string.IsNullOrEmpty(text)) return 0;
                int chars, lines;
                g.MeasureString(text, font, new SizeF(width, 10000), Theme.Wrap, out chars, out lines);
                return Math.Max(1, lines);
            }

            protected static void Frame(Graphics g, RectangleF r)
            {
                using (var shadow = Theme.Round(new RectangleF(r.X - 4, r.Y + 10, r.Width + 8, r.Height + 8), 16))
                using (var b = new SolidBrush(Theme.Shadow)) g.FillPath(b, shadow);
                Theme.Box(g, r, 12, Theme.Panel, Theme.Edge);
            }

            protected static void PaintButtons(MainForm form, Graphics g, List<DialogButton> buttons, RectangleF r, float y)
            {
                // One button sits in the middle; more line up on the right.
                float x = r.Right - 28;
                if (buttons.Count == 1)
                    x = r.X + r.Width / 2 + Math.Max(buttons[0].Primary ? 200 : 130, Theme.Width(g, buttons[0].Label, Theme.Display(18)) + 60) / 2;
                for (int i = buttons.Count - 1; i >= 0; i--)
                {
                    var button = buttons[i];
                    float w = Math.Max(button.Primary ? 200 : 130, Theme.Width(g, button.Label, Theme.Display(18)) + 60);
                    var b = new RectangleF(x - w, y, w, 48);
                    string id = "button" + i;
                    bool on = form.Hover(id);
                    if (button.Primary)
                        using (var p = Theme.Round(b, 5))
                        using (var fill = new LinearGradientBrush(b, on ? Theme.AccentHover : Theme.AccentTop, Theme.AccentBottom, 90f))
                            g.FillPath(fill, p);
                    else Theme.Box(g, b, 5, on ? Theme.ButtonHover : Theme.Button, on ? Theme.Accent : Theme.Edge);
                    Theme.Draw(g, button.Label, Theme.Display(18), button.Primary ? Theme.AccentInk : Theme.Text, b, Theme.Centre);
                    form.AddHit(b, id, button.Click);
                    x -= w + 12;
                }
            }

            public virtual bool Key(MainForm form, Keys key)
            {
                if (key == Keys.Escape && Buttons.Count > 0) { (Buttons.FirstOrDefault(b => !b.Primary) ?? Buttons[0]).Click(); return true; }
                if (key == Keys.Enter && Buttons.Count > 0) { (Buttons.FirstOrDefault(b => b.Primary) ?? Buttons[0]).Click(); return true; }
                return false;
            }

            public virtual void Wheel(MainForm form, int delta) { }
            public virtual void DoubleClick(MainForm form, string id) { }
            public virtual void Closed() { }
        }

        /// <summary>Settings: the text size and dark or light, both shown at once and kept
        /// in settings.txt.</summary>
        private sealed class SettingsDialog : Dialog
        {
            public Action<int, bool> Change;

            public override void Paint(MainForm form, Graphics g)
            {
                float segH = Math.Max(48, Theme.T(38)), labelH = Theme.T(18);
                float h = 26 + Theme.T(34) + 14 + (labelH + 10 + segH) * 2 + 24 + 28 + 48 + 24;
                var r = new RectangleF((CanvasW - Width) / 2, (CanvasH - h) / 2, Width, h);
                Frame(g, r);
                float y = r.Y + 26;
                Theme.Draw(g, Title, Theme.Display(26), Theme.Text, new RectangleF(r.X + 28, y, r.Width - 56, Theme.T(34)));
                y += Theme.T(34) + 14;

                int size = Array.IndexOf(Places.TextScales, Theme.TextScale);
                y = Section(form, g, r, y, "TEXT SIZE", new[] { "Small", "Medium", "Large" }, size, labelH, segH,
                    i => Change(i, Theme.Light), null) + 24;
                Section(form, g, r, y, "APPEARANCE", new[] { "Dark", "Light" }, Theme.Light ? 1 : 0, labelH, segH,
                    i => Change(size, i == 1), (i, box, ink) => { if (i == 0) Moon(g, box, ink); else Sun(g, box, ink); });
                PaintButtons(form, g, Buttons, r, r.Bottom - 24 - 48);
            }

            // A spaced label, then a row of choices with the picked one lit.
            private static float Section(MainForm form, Graphics g, RectangleF r, float y, string label, string[] choices, int picked,
                float labelH, float segH, Action<int> pick, Action<int, RectangleF, Color> icon)
            {
                Font head = Theme.Display(14);
                float tx = r.X + 28;
                foreach (char c in label)
                {
                    Theme.DrawAt(g, c.ToString(), head, Theme.Accent, tx, y);
                    tx += Theme.Width(g, c.ToString(), head) + (c == ' ' ? 6 : 3);
                }
                y += labelH + 10;
                float w = (r.Width - 56 - 10 * (choices.Length - 1)) / choices.Length;
                Font font = Theme.Display(18);
                for (int i = 0; i < choices.Length; i++)
                {
                    var b = new RectangleF(r.X + 28 + i * (w + 10), y, w, segH);
                    string id = label + i;
                    bool on = i == picked, lit = form.Hover(id);
                    if (on)
                        using (var p = Theme.Round(b, 5))
                        using (var fill = new LinearGradientBrush(b, Theme.AccentTop, Theme.AccentBottom, 90f))
                            g.FillPath(fill, p);
                    else Theme.Box(g, b, 5, lit ? Theme.ButtonHover : Theme.Button, lit ? Theme.Accent : Theme.Edge);
                    Color ink = on ? Theme.AccentInk : Theme.Text;
                    if (icon == null) Theme.Draw(g, choices[i], font, ink, b, Theme.Centre);
                    else
                    {
                        // The moon or the sun before the word, the pair centred.
                        float iconS = Math.Max(20, Theme.T(20)), textW = Theme.Width(g, choices[i], font);
                        float x = b.X + (b.Width - iconS - 10 - textW) / 2;
                        icon(i, new RectangleF(x, b.Y + (b.Height - iconS) / 2, iconS, iconS), on ? Theme.AccentInk : Theme.Gold);
                        Theme.Draw(g, choices[i], font, ink, new RectangleF(x + iconS + 10, b.Y, textW + 6, b.Height));
                    }
                    int n = i;
                    form.AddHit(b, id, () => pick(n));
                }
                return y + segH;
            }

            // The Save Editor's crescent: a disc less a disc offset up and right.
            private static void Moon(Graphics g, RectangleF b, Color colour)
            {
                using (var path = new GraphicsPath())
                using (var cut = new GraphicsPath())
                using (var region = new Region())
                {
                    float d = b.Width * 0.86f;
                    path.AddEllipse(b.X + (b.Width - d) / 2, b.Y + (b.Height - d) / 2, d, d);
                    float c = d * 0.78f;
                    cut.AddEllipse(b.X + (b.Width - d) / 2 + d * 0.38f, b.Y + (b.Height - d) / 2 - d * 0.16f, c, c);
                    region.MakeEmpty();
                    region.Union(path);
                    region.Exclude(cut);
                    using (var brush = new SolidBrush(colour)) g.FillRegion(brush, region);
                }
            }

            // The sun: a disc with eight short rays.
            private static void Sun(Graphics g, RectangleF b, Color colour)
            {
                float cx = b.X + b.Width / 2, cy = b.Y + b.Height / 2, r = b.Width * 0.2f;
                using (var brush = new SolidBrush(colour)) g.FillEllipse(brush, cx - r, cy - r, r * 2, r * 2);
                using (var pen = new Pen(colour, Math.Max(1.5f, b.Width / 12)) { StartCap = LineCap.Round, EndCap = LineCap.Round })
                    for (int i = 0; i < 8; i++)
                    {
                        double a = i * Math.PI / 4;
                        float inner = b.Width * 0.32f, outer = b.Width * 0.47f;
                        g.DrawLine(pen, cx + (float)Math.Cos(a) * inner, cy + (float)Math.Sin(a) * inner, cx + (float)Math.Cos(a) * outer, cy + (float)Math.Sin(a) * outer);
                    }
            }
        }

        private Dialog Settings()
        {
            var d = new SettingsDialog { Title = "Settings", Width = 600 };
            d.Change = SetLook;
            d.Buttons.Add(new DialogButton("Close", true, () => CloseDialog(null)));
            return d;
        }

        /// <summary>The faces of a collection or the styles of a variable font, each in its
        /// own letters, to pick one for a tile.</summary>
        private sealed class PickerDialog : Dialog
        {
            public Tile Tile;
            public List<FontFaceChoice> Faces;
            public int Selected, Top;
            public readonly Dictionary<int, LoadedFace> Loaded = new Dictionary<int, LoadedFace>();
            // One sample a face, made again at a new height (a drag of the window's edge left
            // one per height behind until 2026-11-08).
            public readonly Dictionary<int, KeyValuePair<string, Bitmap>> Samples = new Dictionary<int, KeyValuePair<string, Bitmap>>();
            public string Sample;
            public Action<int> Use;
            private const int Visible = 6;
            // A row: the face's name over a sample of its letters, 34 high.
            private static float RowH { get { return 12 + Theme.T(20) + 34 + 10; } }

            public override void Paint(MainForm form, Graphics g)
            {
                int rows = Math.Min(Visible, Faces.Count);
                float listH = rows * RowH + (rows - 1) * 8;
                float h = 26 + Theme.T(34) + 8 + Theme.T(24) + 18 + listH + 22 + 48 + 24;
                var r = new RectangleF((CanvasW - Width) / 2, (CanvasH - h) / 2, Width, h);
                Frame(g, r);
                float y = r.Y + 26;
                Theme.Draw(g, Title, Theme.Display(26), Theme.Text, new RectangleF(r.X + 28, y, r.Width - 56, Theme.T(34)));
                y += Theme.T(34) + 8;
                Font plain = Theme.Body(15.5f), strong = Theme.Body(15.5f, true);
                string a = "Pick the one to use for ", c = ". Each is shown in its own letters.";
                float x = r.X + 28;
                Theme.DrawAt(g, a, plain, Theme.Muted, x, y);
                // The trailing space measured on its own (see MainForm.Rich).
                x += Theme.Width(g, a.TrimEnd(' '), plain) + Theme.Width(g, "a a", plain) - Theme.Width(g, "aa", plain);
                Theme.DrawAt(g, Tile.Def.Name, strong, Theme.Text, x, y);
                x += Theme.Width(g, Tile.Def.Name, strong);
                Theme.DrawAt(g, c, plain, Theme.Muted, x, y);
                y += Theme.T(24) + 18;

                var list = new RectangleF(r.X + 28, y, r.Width - 56 - (Faces.Count > Visible ? 14 : 0), listH);
                for (int i = 0; i < rows; i++)
                {
                    int n = Top + i;
                    var row = new RectangleF(list.X, y + i * (RowH + 8), list.Width, RowH);
                    bool on = n == Selected, lit = form.Hover("row" + n);
                    Theme.Box(g, row, 7, on ? Theme.Selected : lit ? Theme.ButtonHover : Theme.Window, on ? Theme.Accent : Theme.TileEdge);
                    var dot = new RectangleF(row.X + 16, row.Y + (RowH - 18) / 2, 18, 18);
                    using (var pen = new Pen(on ? Theme.Accent : Theme.Radio, 2)) g.DrawEllipse(pen, dot);
                    if (on) using (var b = new SolidBrush(Theme.Accent)) g.FillEllipse(b, dot.X + 4.5f, dot.Y + 4.5f, 9, 9);
                    Theme.Draw(g, Faces[n].Name, Theme.Display(16), on ? Theme.Accent : Theme.Soft, new RectangleF(row.X + 50, row.Y + 9, row.Width - 66, Theme.T(20)));
                    Bitmap sample = SampleFor(n, 34 * form.scale);
                    if (sample != null)
                    {
                        float room = (row.Width - 66) * form.scale;
                        if (sample.Width > room)
                        {
                            using (Region clip = g.Clip)
                            {
                                g.SetClip(new RectangleF(row.X + 50, row.Y, row.Width - 62, row.Height));
                                form.DrawDevice(g, sample, row.X + 49, row.Y + 12 + Theme.T(20));
                                g.Clip = clip;
                            }
                        }
                        else form.DrawDevice(g, sample, row.X + 49, row.Y + 12 + Theme.T(20));
                    }
                    form.AddHit(row, "row" + n, () => { Selected = n; form.Invalidate(); });
                }
                if (Faces.Count > Visible)
                {
                    var rail = new RectangleF(list.Right + 6, y, 4, listH);
                    Theme.Box(g, rail, 2, Theme.TileEdge, Color.Empty);
                    float thumbH = listH * Visible / Faces.Count, thumbY = y + (listH - thumbH) * Top / (Faces.Count - Visible);
                    Theme.Box(g, new RectangleF(rail.X - 1, thumbY, 6, thumbH), 3, Theme.Accent, Color.Empty);
                }
                PaintButtons(form, g, Buttons, r, y + listH + 22);
            }

            private Bitmap SampleFor(int n, float height)
            {
                string key = (int)height + (Theme.Light ? "l" : "d");
                KeyValuePair<string, Bitmap> kept;
                if (Samples.TryGetValue(n, out kept))
                {
                    if (kept.Key == key) return kept.Value;
                    if (kept.Value != null) kept.Value.Dispose();
                }
                Bitmap b;
                try
                {
                    LoadedFace face;
                    if (!Loaded.TryGetValue(n, out face)) Loaded[n] = face = FontFiles.Load(Faces[n]);
                    b = Atlas.Line(face, Sample, height, Theme.Light ? Color.FromArgb(13, 26, 43) : Color.FromArgb(200, 226, 250));
                }
                catch (Exception) { b = null; }
                Samples[n] = new KeyValuePair<string, Bitmap>(key, b);
                return b;
            }

            public override void Wheel(MainForm form, int delta)
            {
                Top = Math.Max(0, Math.Min(Faces.Count - Visible, Top - Math.Sign(delta)));
            }

            public Action Cancel;

            public override bool Key(MainForm form, Keys key)
            {
                if (key == Keys.Escape) { Cancel(); return true; }
                if (key == Keys.Down || key == Keys.Up)
                {
                    Selected = Math.Max(0, Math.Min(Faces.Count - 1, Selected + (key == Keys.Down ? 1 : -1)));
                    if (Selected < Top) Top = Selected;
                    if (Selected >= Top + Visible) Top = Selected - Visible + 1;
                    return true;
                }
                return base.Key(form, key);
            }

            public override void DoubleClick(MainForm form, string id)
            {
                if (id.StartsWith("row")) Use(int.Parse(id.Substring(3)));
            }

            public override void Closed()
            {
                foreach (var face in Loaded.Values) face.Dispose();
                foreach (var b in Samples.Values) if (b.Value != null) b.Value.Dispose();
            }
        }

        private Dialog Picker(Tile tile, List<FontFaceChoice> faces, bool reopen)
        {
            bool styles = faces.Any(f => f.Axes != null);
            var picker = new PickerDialog
            {
                Width = 760,
                Tile = tile,
                Faces = faces,
                Title = System.IO.Path.GetFileName(faces[0].Path) + " holds " + faces.Count + (styles ? " styles" : " fonts"),
                Sample = string.Join(" ", tile.Def.Lines.Select(l => string.Join(" ", l.Select(p => p.Text)))),
            };
            if (reopen && tile.Choice != null)
            {
                int at = faces.FindIndex(f => f.FaceIndex == tile.Choice.FaceIndex && f.Name == tile.Choice.Name);
                if (at >= 0) { picker.Selected = at; picker.Top = Math.Max(0, Math.Min(faces.Count - 6, at - 2)); }
            }
            picker.Use = n => { var f = faces[n]; CloseDialog(null); Apply(tile, f, faces); };
            picker.Cancel = () => CloseDialog(null);
            picker.Buttons.Add(new DialogButton(reopen ? "Another File\u2026" : "Cancel", false, () =>
            {
                CloseDialog(null);
                if (reopen) Browse(tile);
            }));
            picker.Buttons.Add(new DialogButton("Use this font", true, () => picker.Use(picker.Selected)));
            return picker;
        }
    }
}
