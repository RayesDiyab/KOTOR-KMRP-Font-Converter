// KMRP Font Converter: any font file into KOTOR's game fonts, for KMRP or the original game.
//
// The program's start, its one version number and its embedded files. The window is
// MainForm.cs (with its pop-ups in Dialogs.cs), its colours and type Theme.cs, the tiles
// Tiles.cs, settings and the game's folder Places.cs, writing the fonts Conversion.cs and
// Atlas.cs, reading font files FontSource.cs.
using System;
using System.Diagnostics;
using System.Drawing;
using System.IO;
using System.Linq;
using System.Reflection;
using System.Runtime.InteropServices;
using System.Windows.Forms;

[assembly: AssemblyTitle("KMRP Font Converter")]
[assembly: AssemblyProduct("KMRP Font Converter")]
[assembly: AssemblyDescription("Turns any font into KOTOR's game fonts, for KMRP or the original game")]
[assembly: AssemblyCompany("RaymanGT")]
[assembly: AssemblyCopyright("Copyright (c) 2026 RaymanGT. GPL-3.0-or-later.")]
[assembly: AssemblyVersion(KmrpFontConverter.Program.Version + ".0")]
[assembly: AssemblyFileVersion(KmrpFontConverter.Program.Version + ".0")]
[assembly: AssemblyInformationalVersion(KmrpFontConverter.Program.Version)]

namespace KmrpFontConverter
{
    internal static class Program
    {
        /// <summary>The one version number: the footer, the file's properties and the docs
        /// read it from here (audit R1). Raised for every release.</summary>
        public const string Version = "1.0.0";

        [STAThread]
        private static int Main()
        {
            try { SetProcessDPIAware(); } catch (Exception) { }
            FontFiles.CleanTemp();
            Application.EnableVisualStyles();
            Application.SetCompatibleTextRenderingDefault(false);
            var form = new MainForm(Environment.GetCommandLineArgs().Skip(1).ToArray());
            Application.Run(form);
            FontFiles.CleanTemp();
            return form.ExitCode;
        }

        [DllImport("user32.dll")]
        private static extern bool SetProcessDPIAware();

        public static Stream Resource(string name)
        {
            return Assembly.GetExecutingAssembly().GetManifestResourceStream(name);
        }

        public static string ResourceText(string name)
        {
            using (var reader = new StreamReader(Resource(name))) return reader.ReadToEnd();
        }

        public static Bitmap ResourceImage(string name)
        {
            var stream = Resource(name);
            return stream == null ? null : new Bitmap(stream);
        }

        public static void Open(string target)
        {
            try { Process.Start(new ProcessStartInfo(target) { UseShellExecute = true }); }
            catch (Exception) { }
        }
    }
}
