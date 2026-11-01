# Changelog

## 0.3.1 — 2026-11-01

Start screen, macOS menu and closing documents.

- **macOS File menu**: New (⌘N, the New document page), Open… (⌘O), **Recent ▸** (the ten most recent documents, then Recent…), **Save** (⌘S; Save As… for a document that was never saved), **Save As…** (⌥⌘S), Close (⌘W), **Print…** (⌘P, the system print dialog). Save, Save As… and Print… work in a Kadee Markdown tab and are greyed out elsewhere. **Help › License Agreement ▸ MIT / Apache-2.0** shows the license inside KadeeStudio.
- **Closing like macOS Preview**: a new document that was never saved asks *Do you want to keep this new document?* — Save As (name), Where (folder), Format (Markdown file or template), and **Delete** / **Cancel** / **Save**. A document that has a file closes without a question; changes that were not saved are dropped.
- **Start screen sidebar**: Home, **New**, **Open…**, Recent, Starred, Crafting Apps, the KadeeStudio folder, Working locally, Settings. KaDee AI left the sidebar.
- **New** opens the New document page in the middle pane instead of a popup (Cmd+N, then a letter, still works).
- **Open…** has a File type choice: All supported documents, Markdown, PDF, Office, Graphics, Media, CAD, Dashboard, Report (a File type menu under the macOS Open panel).
- **Document rows as in GenOffice**: a **…** button after the star (on hover, and on right-click) opens *Open*, *Show in folder*, *Copy path*, *Rename…*, *Duplicate*, *Remove from list* (*Unstar* on Starred) and **Delete files**, which asks *Move “x” to the Trash?* first. Clicking a file that cannot be found asks *File unavailable … Remove it from the list?*. **Starred** drops files that no longer exist.
- **Folders, as in GenOffice**: the sidebar's FOLDERS is a folder tree — the KadeeStudio folder, then the folders you add with **Add folder…** (kept in `home.json`, nothing is copied or moved). Chevrons open folders to show the folders inside (what is open is remembered); a folder that is gone shows *Not available*. Clicking a folder shows it in the middle pane: its path as clickable steps, the quick-start cards (new documents go into that folder), its sub-folders first and then its documents, or *This folder is empty*. The **…** button and the right-click menu offer *New subfolder*, *Rename…*, *Show in folder*, *Delete folder…* (to the Trash, after asking) and *Remove from list* for added folders; **+** makes a new folder in the folder on screen. Recent and Starred follow a renamed folder.
- The documents folder is shown under its own name everywhere on the start screen (the folder tree, its page, the New document page and Settings): change it to `~/Work` and it reads *Work*.
- **Crafting Apps**: TextureCraft (3D texture painting) joins, with its icon and `.texturecraft` files.
- **Settings** opens in the middle pane instead of a popup. General has two new options, *Open each module in a new Tab/Window* and *Open each file in a new Tab/Window*; separate windows come in a later version, so Tab is the choice for now (an open document is focused, not opened twice). New sections **License — MIT** and **License — Apache-2.0**; *MIT* and *Apache-2.0* in About link to them.
- **KaDee AI** opens as a pane on the right everywhere, also on Home (a general chat there), from the ✦ button, Cmd+J or Cmd+Shift+A.
- New commands with shortcuts: `file.save` (Cmd+S), `file.save_as` (Option+Cmd+S), `file.print` (Cmd+P), plus `library.delete` and `home.show.new`. If you changed shortcuts before, add them in Settings › Keyboard (or Reset to defaults).
- The Crafting Apps tiles on Home keep their size and wrap to as many per row as fit, so the rows rearrange when the window narrows or the KaDee AI pane opens.
- Fixes: the editor of a closed tab no longer stays on screen (Linux); the document list puts the search box on its own row when the KaDee AI pane makes it narrow; no unused-import warning on macOS.

## 0.3.0 — 2026-11-01

Phase 2: Kadee Markdown.

- **Kadee Markdown**: mdcraft's Markdown word processor in the Markdown tab, with all of its functionality — the ribbon (Home, Insert, Design, Slides, Review, View, Table), the editor and live preview with diagrams, charts, maths and media, slides and the F5 slide show, the File view with 19 templates and My templates, export (PDF, web page, formatted text), printing, crash recovery and Options.
- mdcraft's crates are part of the workspace (`crates/mdcraft-core`, `crates/mdcraft-templates`) **without mdcraft's AI code**: the agent bridge (`agent.rs`, `agent.js`, `--control`) and the MCP server are left out. KaDee AI is the assistant.
- **KaDee AI and Markdown**: the AI pane of a Markdown tab reads the editor's current text, unsaved edits included, and each answer has *Insert into document* (inserted at the cursor, one undo step).
- The page's requests are answered by KadeeStudio: Open and Save As dialogs, printing, full screen, closing with unsaved changes (closing a tab or the window asks first). A template or a copy that mdcraft would open in a new window opens in a new tab; Save As moves the tab to the new file.
- KadeeStudio shortcuts work while the editor has the keyboard (Cmd+W, Ctrl+Tab, Cmd+N, Cmd+J, Cmd+1…9 …); the macOS Edit menu acts on the editor. Menus, the New document menu and Settings show above the editor.
- The tab shows the document's name with • while it has unsaved changes.
- New documents and dialogs use the KadeeStudio documents folder; My templates live in KadeeStudio's `Templates` folder; mdcraft's settings in `Markdown` next to `home.json`.
- Inside the editor the name is Kadee Markdown; About credits mdcraft and MarkDown Live.
- Linux: needs WebKitGTK 4.1 (`libwebkit2gtk-4.1-dev` to build, `libwebkit2gtk-4.1-0` to run; added to CI, the release workflow and the .deb/.rpm dependencies). KadeeStudio runs on X11/XWayland so the editor can be embedded (`KADEE_KEEP_WAYLAND=1` keeps native Wayland, without the editor).
- Windows: Markdown files keep the file tab for now (mdcraft runs on macOS and Linux).

## 0.2.0 — 2026-11-01

Phase 1: KaDee AI.

- **KaDee AI**, ported from SolarOffice's AI layer into a new crate, `kadee-ai`: one API over Anthropic, Gemini, OpenAI, Mistral, DeepSeek, Qwen, Kimi, GLM, Cohere, Meta, Microsoft Foundry, Grok, Aleph Alpha, Sakana, local Ollama and LM Studio, OpenAI-compatible endpoints and custom providers. Streaming, retries on rate limits and overloads, usage and cost per answer, a model catalogue with prices.
- **KaDee AI page** (sidebar, `Cmd+Shift+A`): streaming chat with Stop, New chat, model picker, *For* a module, and starred files as context.
- **AI pane** next to every open file (✦ in the tab strip, `Cmd+J`): knows the file and uses the module's instructions and model.
- **Settings › AI providers** (by region, status chips, Test connection, add, edit and remove providers, custom providers) and **Settings › AI models** (default model, one model per part of KadeeStudio, add your own models). AI changes are saved on Done and dropped on Cancel; they live in `ai.toml` next to `home.json`.
- New commands with stable ids: `home.show.ai`, `ai.toggle_pane`, `settings.open.ai-providers`, `settings.open.ai-models`.
- `--page=ai`, `--page=ai-providers` and `--page=ai-models` open them from the command line.
- If you changed shortcuts in 0.1.0, the new KaDee AI shortcuts are not in your saved key map: add them in Settings › Keyboard (or Reset to defaults).

## 0.1.0 — 2026-10-31

Phase 0: the start screen.

- Native Rust desktop shell (egui/eframe): tab strip with Home and one tab per file, sidebar, light/dark/system theme.
- Quick-start tiles for Markdown, PDF, Dashboard and Report, plus Open file.
- Documents table with filter pills (All, Office, Markdown, PDF, Dashboard, Report, Graphics, Media, CAD), search, sorting, stars and a right-click menu. **All** lists every file type KadeeStudio or a Crafting App opens.
- Crafting Apps strip and page, alphabetically: CADCraft, DeckCraft, DesignCraft, EffectCraft, FigCraft, FilmCraft, GridCraft, LightCraft, PdfCraft, PhotoCraft, SolidCraft, SoundCraft, VectorCraft, WordCraft.
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
