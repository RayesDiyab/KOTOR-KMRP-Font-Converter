// KMRP Font Converter: the settings file, and where the game and its Override folder are.
using System;
using System.Collections.Generic;
using System.IO;
using System.Linq;
using Microsoft.Win32;

namespace KmrpFontConverter
{
    internal static class Places
    {
        /// <summary>Where the last font was picked from, for the next pick (this run only).</summary>
        public static string LastFontFolder;

        private static string SettingsFile
        {
            get { return Path.Combine(Environment.GetFolderPath(Environment.SpecialFolder.ApplicationData), "KMRP Font Converter", "settings.txt"); }
        }

        // settings.txt: key=value lines. output (the folder), textsize (small, medium,
        // large), theme (dark, light) and version (kmrp, original).
        private static Dictionary<string, string> Settings()
        {
            var values = new Dictionary<string, string>();
            try
            {
                if (File.Exists(SettingsFile))
                    foreach (string line in File.ReadAllLines(SettingsFile))
                    {
                        int eq = line.IndexOf('=');
                        if (eq > 0) values[line.Substring(0, eq)] = line.Substring(eq + 1);
                    }
            }
            catch (Exception) { }
            return values;
        }

        public static string Setting(string key)
        {
            string value;
            return Settings().TryGetValue(key, out value) ? value : null;
        }

        public static void Set(string key, string value)
        {
            try
            {
                var values = Settings();
                values[key] = value;
                Directory.CreateDirectory(Path.GetDirectoryName(SettingsFile));
                File.WriteAllLines(SettingsFile, values.Select(v => v.Key + "=" + v.Value));
            }
            catch (Exception) { }
        }

        public static string SavedFolder()
        {
            string folder = Setting("output");
            try { return folder != null && Directory.Exists(folder) ? folder : null; }
            catch (Exception) { return null; }
        }

        public static void Save(string folder) { Set("output", folder); }

        public static readonly string[] TextSizes = { "small", "medium", "large" };
        public static readonly float[] TextScales = { 1f, 1.2f, 1.32f };

        /// <summary>The text size and theme from settings.txt, into Theme.</summary>
        public static void LoadLook()
        {
            int size = Array.IndexOf(TextSizes, Setting("textsize"));
            Theme.TextScale = TextScales[size < 0 ? 1 : size];
            Theme.Light = Setting("theme") == "light";
        }

        /// <summary>The version the fonts are made for: as last chosen, else KMRP when KMRP is
        /// installed in the game the output folder belongs to (or, with no game found there,
        /// in any game found), else the original game.</summary>
        public static Target LoadTarget(string output)
        {
            string saved = Setting("version");
            if (saved == "kmrp") return Target.Kmrp;
            if (saved == "original") return Target.Original;
            try
            {
                string game = Path.GetDirectoryName(output.TrimEnd('\\', '/'));
                if (game != null && File.Exists(Path.Combine(game, "swkotor.exe"))) return HasKmrp(game) ? Target.Kmrp : Target.Original;
                return Games().Any(g => File.Exists(Path.Combine(g, "swkotor.exe")) && HasKmrp(g)) ? Target.Kmrp : Target.Original;
            }
            catch (Exception) { return Target.Original; }
        }

        public static void SaveTarget(Target target) { Set("version", target == Target.Kmrp ? "kmrp" : "original"); }

        // KMRP's installer leaves its folder and its module in the game's folder.
        private static bool HasKmrp(string game)
        {
            return File.Exists(Path.Combine(game, "patches", "kmrp.dll")) || Directory.Exists(Path.Combine(game, "KMRP"));
        }

        /// <summary>Whether `folder` is a game's Override folder (beside swkotor.exe).</summary>
        public static bool IsOverride(string folder)
        {
            try
            {
                string trimmed = folder.TrimEnd('\\', '/');
                return string.Equals(Path.GetFileName(trimmed), "Override", StringComparison.OrdinalIgnoreCase) &&
                       File.Exists(Path.Combine(Path.GetDirectoryName(trimmed), "swkotor.exe"));
            }
            catch (Exception) { return false; }
        }

        /// <summary>The game's Override folder when the game can be found (Steam or GOG),
        /// else a folder in Documents.</summary>
        public static string DefaultFolder()
        {
            foreach (string game in Games())
                if (File.Exists(Path.Combine(game, "swkotor.exe"))) return Path.Combine(Cased(game), "Override");
            return Path.Combine(Environment.GetFolderPath(Environment.SpecialFolder.MyDocuments), "KMRP Font Converter");
        }

        // A folder's name as the disk spells it (Steam keeps its own path in lower case).
        private static string Cased(string path)
        {
            try
            {
                var dir = new DirectoryInfo(path);
                if (dir.Parent == null) return dir.FullName.ToUpperInvariant();
                var match = dir.Parent.GetDirectories(dir.Name);
                return Path.Combine(Cased(dir.Parent.FullName), match.Length > 0 ? match[0].Name : dir.Name);
            }
            catch (Exception) { return path; }
        }

        private static IEnumerable<string> Games()
        {
            var found = new List<string>();
            // The folder the converter itself is in, and the one above it: a player who puts
            // it in the game's folder (a disc or other install no registry key names) has the
            // game found that way.
            try
            {
                string here = AppDomain.CurrentDomain.BaseDirectory.TrimEnd('\\');
                found.Add(here);
                string up = Path.GetDirectoryName(here);
                if (up != null) found.Add(up);
            }
            catch (Exception) { }
            try
            {
                string steam = Registry.GetValue(@"HKEY_CURRENT_USER\Software\Valve\Steam", "SteamPath", null) as string;
                if (!string.IsNullOrEmpty(steam))
                {
                    var libraries = new List<string> { steam.Replace('/', '\\') };
                    string vdf = Path.Combine(steam, "steamapps", "libraryfolders.vdf");
                    if (File.Exists(vdf))
                        foreach (string line in File.ReadAllLines(vdf))
                        {
                            string t = line.Trim();
                            if (!t.StartsWith("\"path\"")) continue;
                            string[] parts = t.Split('"');
                            if (parts.Length >= 4) libraries.Add(parts[3].Replace(@"\\", @"\"));
                        }
                    foreach (string library in libraries) found.Add(Path.Combine(library, "steamapps", "common", "swkotor"));
                }
            }
            catch (Exception) { }
            foreach (string key in new[] { @"HKEY_LOCAL_MACHINE\SOFTWARE\WOW6432Node\GOG.com\Games\1207666283", @"HKEY_LOCAL_MACHINE\SOFTWARE\GOG.com\Games\1207666283" })
                try
                {
                    string gog = Registry.GetValue(key, "path", null) as string;
                    if (!string.IsNullOrEmpty(gog)) found.Add(gog);
                }
                catch (Exception) { }
            found.Add(@"C:\Program Files (x86)\Steam\steamapps\common\swkotor");
            found.Add(@"C:\GOG Games\Star Wars - KotOR");
            return found;
        }
    }
}
