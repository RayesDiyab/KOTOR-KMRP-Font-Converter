// KMRP Font Converter: writing a set of game fonts into a folder, all or nothing.
//
// Until 2026-11-08 each font was written straight to its final name, the .tga then the .txi,
// and a .tpc of the same name moved aside as each came up. A failure half way (a read-only
// file of another mod, a full disk, the window closed) left some fonts new and some old in
// the game's Override, and could leave a new .tga beside an old .txi (audit F1 to F3). Now
// every font is made in a folder of its own inside the output folder first; only when all of
// them are made, and every file they replace can be replaced, are they moved into place, one
// rename each.
using System;
using System.Collections.Generic;
using System.IO;
using System.Linq;

namespace KmrpFontConverter
{
    internal sealed class Conversion
    {
        public string Folder;
        public Target Target;
        public Dictionary<string, Reference> References;
        /// <summary>Each game font name and the face it is made from.</summary>
        public List<KeyValuePair<string, LoadedFace>> Work = new List<KeyValuePair<string, LoadedFace>>();
        /// <summary>Told after each font is made: how many are done.</summary>
        public Action<int> Progress;
        /// <summary>Asked between fonts: true stops the conversion with nothing written.</summary>
        public Func<bool> Stop;
        /// <summary>Set once files start to move into the folder: a failure after it may have
        /// left some fonts new and some old there.</summary>
        public bool Moving;

        public const string TpcBackup = ".before-font-converter";

        /// <summary>How many font files the folder already has under the names these fonts
        /// use: what Convert asks about before replacing them.</summary>
        public static int Existing(string folder, IEnumerable<string> names)
        {
            int existing = 0;
            try
            {
                foreach (string n in names)
                    foreach (string ext in new[] { ".tga", ".txi", ".tpc" })
                        if (File.Exists(Path.Combine(folder, n + ext))) existing++;
            }
            catch (Exception) { }
            return existing;
        }

        /// <summary>Makes every font, then puts them in place. Throws with a message for the
        /// player when it cannot; then nothing in the folder has changed, unless the moves
        /// themselves fail, which the message says.</summary>
        public void Run()
        {
            Directory.CreateDirectory(Folder);
            string work = Path.Combine(Folder, ".kmrp-font-converter-" + Guid.NewGuid().ToString("N").Substring(0, 8));
            Directory.CreateDirectory(work);
            try
            {
                int done = 0;
                foreach (var item in Work)
                {
                    if (Stop != null && Stop()) throw new OperationCanceledException();
                    Atlas.Write(item.Value, References[item.Key], work, Target);
                    if (Progress != null) Progress(++done);
                }
                if (Stop != null && Stop()) throw new OperationCanceledException();
                var names = Work.Select(w => w.Key).Distinct().ToList();
                // Everything that will be replaced must be replaceable before anything moves.
                foreach (string name in names)
                    foreach (string ext in new[] { ".tga", ".txi", ".tpc" })
                    {
                        string path = Path.Combine(Folder, name + ext);
                        if (File.Exists(path) && (File.GetAttributes(path) & FileAttributes.ReadOnly) != 0)
                            throw new IOException(name + ext + " in the folder is read-only, so it can't be replaced. Nothing was changed.");
                    }
                Moving = true;
                foreach (string name in names)
                {
                    // A .tpc of the same name would win over the new .tga: set aside, under a
                    // name no earlier conversion took.
                    string tpc = Path.Combine(Folder, name + ".tpc");
                    if (File.Exists(tpc)) File.Move(tpc, Free(tpc + TpcBackup));
                    foreach (string ext in new[] { ".tga", ".txi" })
                    {
                        string target = Path.Combine(Folder, name + ext);
                        if (File.Exists(target)) File.Delete(target);
                        File.Move(Path.Combine(work, name + ext), target);
                    }
                }
            }
            finally
            {
                try { Directory.Delete(work, true); } catch (Exception) { }
            }
        }

        // `path`, or path-2, path-3... when it is taken (audit F2: a second conversion stopped
        // at a .tpc already set aside).
        private static string Free(string path)
        {
            if (!File.Exists(path)) return path;
            for (int n = 2; ; n++)
                if (!File.Exists(path + "-" + n)) return path + "-" + n;
        }
    }
}
