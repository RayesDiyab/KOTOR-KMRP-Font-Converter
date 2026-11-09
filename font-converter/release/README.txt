KMRP Font Converter 1.0.0
=========================

Turns any font file into Star Wars: Knights of the Old Republic's game fonts: the
menus, the dialogue, the journal, the HUD's numbers and the end credits, each in the
font you pick. For KMRP (the KOTOR Modern Restoration Patch) or for the original game.

Made by RaymanGT.
The converter:  https://github.com/RayesDiyab/KOTOR-KMRP-Font-Converter
KMRP:           https://github.com/RayesDiyab/KMRP-KOTOR-Modern-Restoration-Patch


What you need
-------------

- One of:
    Windows 10 or 11, 64-bit       KOTOR-Font-Converter-<version>-Windows-x64.zip
    macOS 11 or newer, Apple       KOTOR-Font-Converter-<version>-macOS.dmg (or .zip): one
      silicon or Intel             app for both
    Linux, 64-bit Intel or AMD     KOTOR-Font-Converter-<version>-Linux-x86_64.AppImage
    Linux, 64-bit ARM              KOTOR-Font-Converter-<version>-Linux-aarch64.AppImage
  A graphics driver with OpenGL 2 or newer, as every desktop has.
- KOTOR 1. For the KMRP version, KMRP 1.6.0 or later. On macOS and Linux the game is the
  Windows game run through Steam's Proton, CrossOver or Wine: the fonts are for its Override
  folder. (KMRP is for the Windows game.)


How to use it
-------------

1. Start it. Nothing is installed.
     Windows  "KOTOR-Font-Converter-<version>-Windows-x64.exe"
     macOS    open the DMG and drag "KOTOR-Font-Converter-<version>-macOS" to
              Applications, then open it.
              It is not notarized by Apple: the first time, right-click it and choose Open.
     Linux    make the AppImage executable (chmod +x, or the file's Properties) and run it.
2. On a tile, click "Choose a font..." and pick a font file (TTF, OTF, TTC, OTC,
   WOFF or WOFF2), or drag the file onto the tile. Each tile is one kind of text in
   the game; you do not have to change them all.
3. In the Version box, choose:
     KMRP           the fonts are made at KMRP's HD size, and KMRP sizes them for
                    your screen and your text size;
     Original Game  the fonts are made at the game's own size, for the game without
                    KMRP. The text keeps the game's size, which is small on a modern
                    screen.
   The converter starts on KMRP when it finds KMRP in your game, else on Original Game,
   and remembers your choice.
4. Click Convert. It turns green ("Converted") when the fonts are written.
5. Start the game. With KMRP, open Options > Graphics > Text Size and Font: "Modded
   Font" is chosen by itself when new fonts are in Override; choose HD Font or Original
   Font there to see KMRP's own fonts again without removing yours.


Where the fonts go
------------------

Into the game's Override folder when the converter finds the game (Steam on any system,
GOG on Windows, or the converter put in the game's folder); "Change" picks another folder.
When no game is found they go to Documents/KMRP Font Converter: copy them into the game's
Override folder yourself.

Fonts already in the folder under the same names are replaced (you are asked first).
A .tpc file of the same name is not deleted: it is renamed to end in
.before-font-converter, because the game would use it instead of the new font.


How to undo it
--------------

Delete these files from the Override folder (a .tga and a .txi of each name):

  dialogfont10x10   dialogfont10x10a  dialogfont10x10b  dialogfont12x16
  dialogfont16x16   dialogfont16x16a  dialogfont16x16b  dialogfont32x32
  fnt_console       fnt_credits       fnt_creditsa      fnt_creditsb
  fnt_d10x10b       fnt_d16x16        fnt_d16x16a       fnt_d16x16b
  fnt_dialog16x16

If any file ends in .tpc.before-font-converter, remove that ending to put it back.


Settings
--------

The text size, light or dark, the folder and the version are kept in settings.txt in a
folder "KMRP Font Converter" in the system's settings folder:
  Windows  %APPDATA%\KMRP Font Converter\settings.txt
  macOS    ~/Library/Application Support/KMRP Font Converter/settings.txt
  Linux    ~/.config/KMRP Font Converter/settings.txt
Delete it to start afresh.

The window fits itself to the screen: on a smaller screen it opens smaller, and everything
in it shrinks evenly with it, down to half its full size.


Licence
-------

The KMRP Font Converter is free software under the GNU General Public License,
version 3 or later (LICENSE.txt). Its source is in the KOTOR-KMRP Font Converter
repository above, in the font-converter folder.

The window's text is set in KMRP Original HD Menu and Descriptions, under the SIL Open
Font License 1.1 (OFL.txt). Other credits are in THIRD-PARTY-NOTICES.txt.

Star Wars: Knights of the Old Republic and KOTOR are trademarks of their owners. This
program is made by fans and is not affiliated with them.
