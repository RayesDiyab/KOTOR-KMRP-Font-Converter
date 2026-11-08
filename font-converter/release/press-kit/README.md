# KMRP Font Converter 1.0 press kit

Release art for the KMRP Font Converter 1.0 (released with KMRP 1.6.0): the cover and the
pages for mod pages, posts and articles, in the converter's own style. Use them as they are;
please do not crop a frame out of its page without saying what it shows.

Held to the [documentation standard](../../../docs/documentation-standard.md): where each
frame came from is said below.

Five pages, in the order to show them.

| File | What |
| --- | --- |
| [`01-cover.png`](01-cover.png) | Cover: Font Converter 1.0, its three steps, the font files it reads, four sample tiles (Georgia, Arial, Consolas, KMRP Original HD) |
| [`02-the-converter.png`](02-the-converter.png) | The converter's window, five fonts picked, KMRP chosen, 1600x900 |
| [`03-original-game.png`](03-original-game.png) | Original game: the main menu in the game's own font beside the same menu in Georgia, 1280x960 |
| [`04-conversations.png`](04-conversations.png) | Original game: a conversation and its replies in Georgia, 1280x960 |
| [`05-with-kmrp.png`](05-with-kmrp.png) | With KMRP: Georgia on KMRP's Text Size and Font page, 3440x1440 |

## How they were made

- **Everything is composed by [`make_press_kit.py`](../make_press_kit.py)** in the
  converter's own look, deliberately apart from KMRP's starfield release art: the window's
  navy with faint baselines, the window's colours (orange for a picked font, blue for the
  choice, green for converted), rounded frames edged in those colours, and the two KMRP
  Original HD faces the converter itself uses. It needs no KMRP checkout: this repository,
  Pillow, Windows' own Georgia, Arial and Consolas (the cover's sample tiles) and the
  screenshots below. Run `python font-converter/release/make_press_kit.py`.
- **Text is placed by its drawn ink,** not by the font's line box: step numbers sit on the
  centre of their discs and labels on the middle of their bars by measurement.
- **Nothing inside a frame is retouched or cropped:** each frame is the whole screen or
  window, scaled down, with its own size under it.
- **The window** (page 2) was captured on 2026-11-09 from the 1.0.0 build at 1600x900, with
  `--still` so no hover shows, five tiles picked (Arial, Georgia, Consolas).
- **The original-game frames** (pages 3 and 4) were taken on 2026-11-08 in a copy of the
  game with the untouched GOG 1.03 executable and no KMRP or other patch, at 1280x960 in a
  window: first with the game's own fonts, then with Georgia made by the converter for the
  original game in its Override folder (`docs/font-converter.md`, the vanilla check).
- **The KMRP frame** (page 5) is the maintainer's screenshot of 2026-11-08 at 3440x1440,
  reduced to 2000x837 before it reached this folder, with Georgia made for KMRP in Override.
  It was taken before KMRP's Modded Font row existed, so the page shows HD Font chosen.
- The screenshots themselves are kept out of the repository (`build/press-kit-shots`).
