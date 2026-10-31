# Changelog

## 0.1.0 — 2026-10-31

Phase 0: the start screen.

- Native Rust desktop shell (egui/eframe): tab strip with Home and one tab per file, sidebar, light/dark/system theme.
- Quick-start tiles for Markdown, PDF, Dashboard and Report, plus Open file.
- Documents table with filter pills (All, Office, Markdown, PDF, Dashboard, Report, Graphics, Media, CAD), search, sorting, stars and a right-click menu. **All** lists every file type KadeeStudio or a Crafting App opens.
- Crafting Apps strip and page, alphabetically: CadCraft, DeckCraft, DesignCraft, EffectCraft, FigCraft, FilmCraft, GridCraft, LightCraft, PdfCraft, PhotoCraft, SolidCraft, SoundCraft, VectorCraft, WordCraft.
- Settings popup with General, Keyboard and About. Done keeps changes, Cancel / × / Esc puts them back. About combines the version, build, system, installed Crafting Apps, credits and license; on macOS the menu bar's About KadeeStudio opens it too.
- macOS menu bar like ONLYOFFICE's: KadeeStudio, File, Edit, Window and Help menus, with the shortcuts from the key map shown on the items.
- About: *Source code* opens github.com/kdugardyn66/KaDeeStudio; *Open templates folder* opens `Templates` next to `home.json` (`~/Library/Application Support/be.Kadee.KadeeStudio/Templates` on macOS).
- The Crafting Apps page points to the ArtCraft Launcher only (it includes SolidCraft), plus *Look again*.
- The start screen shows the installed Crafting Apps with their own icons and opens them on click; the Crafting Apps page shows every app as installed or not.
- Keyboard shortcut editor (Settings › Keyboard): record shortcuts by pressing them, key sequences such as `Cmd+N M`, add, clear, remove and reset shortcuts; saved with the settings.
- New document menu: Cmd+N opens it at once, then a letter picks the type — M Markdown, P PDF, D Dashboard, R Report, W Word, G spreadsheet, S slide deck. The letters are editable shortcuts (`Cmd+N M` …). Ctrl+Tab / Ctrl+Shift+Tab switch tabs.
- New PDF documents: a blank A4 page. New Word documents, spreadsheets and slide decks: blank `.docx`, `.xlsx` and `.pptx` files for WordCraft, GridCraft and DeckCraft.
- Modifier keys follow the system: Cmd, Ctrl, Option, Shift on macOS; Win, Ctrl, Alt, Shift on Windows; Super, Ctrl, Alt, Shift on Linux — each a separate key.
- New logo: a K in a grid of module tiles with an AI spark.
- Build scripts: macOS (.app and .dmg, using `diskutil image` on macOS 26+), Linux (.tar.gz, .deb, .rpm, AppImage) and Windows (.zip, .msi); GitHub Actions CI and release workflow.
