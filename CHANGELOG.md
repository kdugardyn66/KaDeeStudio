# Changelog

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
