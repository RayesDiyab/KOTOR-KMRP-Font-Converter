// KMRP Font Converter: the eleven tiles, one for each distinct font the game draws its text in.
using System.Collections.Generic;
using System.Drawing;
using System.Globalization;
using System.Linq;

namespace KmrpFontConverter
{
    internal sealed class Piece
    {
        public string Name, Resref, Text;
        public float Height;
        public Color Colour;
        public Bitmap Hd;        // drawn in the HD Original font at 3x (make_data.py)
    }

    internal sealed class TileDef
    {
        public string Key, Name, Group, Use;
        public string[] Resrefs;
        public bool Centre;
        public List<List<Piece>> Lines = new List<List<Piece>>();
    }

    internal sealed class Tile
    {
        public TileDef Def;
        public FontFaceChoice Choice;
        public LoadedFace Face;
        public List<FontFaceChoice> FileFaces;     // every face of the picked file, for the picker
        public List<char> Missing;
        public bool Converted;                     // written since it was picked
        public readonly Dictionary<string, Bitmap> Cache = new Dictionary<string, Bitmap>();
        public RectangleF Bounds;

        public void Forget()
        {
            foreach (var b in Cache.Values) b.Dispose();
            Cache.Clear();
        }
    }

    internal static class Tiles
    {
        // The game's 17 text fonts hold 11 distinct letter sets (tools/build_hd_original_otf.py
        // finds them by comparing every glyph). A tile is one set; its font is written to
        // every name that shares it, so text the game draws in either name changes together.
        public static List<TileDef> All()
        {
            var defs = new List<TileDef>
            {
                Def("menu", "Menu", "Menus", "Screen titles and buttons", false, "dialogfont16x16", "dialogfont16x16b"),
                Def("menusmall", "Menu Small", "Menus", "Option lists and small labels", false, "dialogfont10x10", "dialogfont10x10a", "dialogfont16x16a"),
                Def("menusmallb", "Menu Small Bold", "Menus", "Tabs and bold labels", false, "dialogfont10x10b"),
                Def("menumedium", "Menu Medium", "Menus", "Save and load lists", false, "dialogfont12x16"),
                Def("menularge", "Menu Large", "Menus", "The biggest headings", true, "dialogfont32x32"),
                Def("dialogue", "Dialogue", "Reading", "What people say, and subtitles", false, "fnt_d16x16", "fnt_d10x10b"),
                Def("descriptions", "Replies", "Reading", "Your answers, item and skill text", false, "fnt_d16x16b"),
                Def("light", "Journal", "Reading", "The journal and long reading", false, "fnt_d16x16a", "fnt_creditsa", "fnt_dialog16x16"),
                Def("console", "HUD", "HUD", "Credits, level and XP", false, "fnt_console"),
                Def("credits", "Credits", "Credits", "Job titles", true, "fnt_credits"),
                Def("creditsb", "Credits Bold", "Credits", "Names", true, "fnt_creditsb"),
            };
            var byKey = defs.ToDictionary(d => d.Key);
            TileDef tile = null;
            foreach (string raw in Program.ResourceText("examples.txt").Split('\n'))
            {
                string line = raw.TrimEnd('\r');
                if (line.StartsWith("tile ")) tile = byKey[line.Substring(5)];
                else if (line.Trim() == "line") tile.Lines.Add(new List<Piece>());
                else if (line.TrimStart().StartsWith("piece "))
                {
                    string[] p = line.Trim().Split(new[] { ' ' }, 8);
                    tile.Lines[tile.Lines.Count - 1].Add(new Piece
                    {
                        Name = p[1], Resref = p[2], Height = float.Parse(p[3], CultureInfo.InvariantCulture),
                        Colour = Color.FromArgb(int.Parse(p[4]), int.Parse(p[5]), int.Parse(p[6])), Text = p[7],
                        Hd = Program.ResourceImage("examples." + p[1] + ".png"),
                    });
                }
            }
            return defs;
        }

        private static TileDef Def(string key, string name, string group, string use, bool centre, params string[] resrefs)
        {
            return new TileDef { Key = key, Name = name, Group = group, Use = use, Centre = centre, Resrefs = resrefs };
        }
    }
}
