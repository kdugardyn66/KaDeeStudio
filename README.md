# KadeeStudio

A native Rust office suite with AI, Markdown, PDF, conversion, dashboards and reports, and real-time collaboration through a self-hosted server on Docker or Kubernetes. It will also host the Crafting Apps: the ten ArtCraft apps and SolidCraft.

**Status — v0.1.0, Phase 0: the start screen.** No editors, no AI, no server yet.

![KadeeStudio start screen](docs/screenshots/home-light.png)

## The start screen

- **Tab strip**: a Home tab, one tab per opened file (middle-click or × to close), a **+** menu for new documents, and a theme switch (System → Light → Dark).
- **Sidebar**: Home, Recent, Starred, Crafting Apps, the KadeeStudio documents folder, a collaboration status card, Settings.
- **Greeting** with your name and today's date.
- **Quick-start tiles** for Markdown, PDF, Dashboard and Report, plus *Open file*. Markdown, Dashboard and Report create a new file in `~/Documents/KadeeStudio` and open it in a tab; PDF opens a PDF. Convert has no tile; it will live under File › Convert and the right-click menu.
- **Crafting Apps** strip and page: WordCraft, GridCraft, DeckCraft, PdfCraft, DesignCraft, PhotoCraft, VectorCraft, LightCraft, FilmCraft, EffectCraft (ArtCraft) and SolidCraft (Tangency Labs); integration in Phase 7.
- **Documents table**: filter pills All, Markdown, PDF, Dashboard, Report, Office, Graphics, Media and 3D. **All** includes every file type KadeeStudio or a Crafting App opens (`.docx`, `.xlsx`, `.pptx`, `.psd`, `.svg`, raw photos, video, `.step`, `.stl` …; the registry is `crates/kadee-core/src/doctype.rs` and `craft.rs`), and each row says which app opens it. Search, sortable Name / Opened / Modified / Size, star on hover, right-click menu (Open, Open in default app, Show in folder, Copy path, Star, Remove from Recent); missing files are marked.
- **File tab**: until the editors exist, it shows the file's details and the app that will open it, with *Open in default app*, *Show in folder* and *Star*.
- **Settings**: your name, theme, documents folder, keyboard shortcuts, about.
- First launch creates `~/Documents/KadeeStudio` with *Welcome to KadeeStudio.md*.
- State is saved in `home.json` in the platform config folder; `KADEE_CONFIG_DIR` overrides it.

### Shortcuts (Cmd on macOS, Ctrl elsewhere)

| Keys | Action |
| --- | --- |
| Ctrl+N | New Markdown document |
| Ctrl+Shift+D / Ctrl+Shift+R | New dashboard / report |
| Ctrl+O | Open a file |
| Ctrl+W, Ctrl+Shift+T | Close tab, reopen closed tab |
| Ctrl+1 … Ctrl+9 | Home, tab 2–9 |
| Ctrl+Tab, Ctrl+Shift+Tab | Next / previous tab |
| Ctrl+, | Settings |

## Build and run

Needs Rust 1.88 or newer.

```sh
cargo run -p kadeestudio                     # the app
cargo run -p kadeestudio -- notes.md a.pdf   # open files in tabs
cargo run -p kadeestudio --features glow     # OpenGL renderer (old GPUs, VMs)
cargo test --workspace
cargo clippy --workspace --all-targets
```

Linux needs the usual windowing libraries (`libxkbcommon`, `libwayland`, X11 or Wayland, a Vulkan or OpenGL driver). A screenshot of the first frame for docs or CI:

```sh
EFRAME_SCREENSHOT_TO=home.png cargo run -p kadeestudio --features screenshot
```

## Layout

```
apps/kadeestudio/        the desktop app (egui / eframe)
  src/main.rs            window, app icon, renderer choice
  src/app.rs             state, the command runner
  src/icons.rs           line icons painted in code
  src/ui/                tab strip + sidebar (mod.rs), home, file table, pages, file tab
crates/kadee-core/       modules, Crafting Apps catalogue, file-type registry, Recent/Starred, settings, home.json, file helpers (no UI)
crates/kadee-ui/         design tokens, light/dark palettes, badges, pills, buttons
crates/kadee-commands/   every action as a Command with a stable id; the key map
```

Rules: `kadee-core` never depends on egui, and every click or shortcut becomes a `Command` that runs in one place (`app.rs`). This keeps the door open for a command palette, CLI, MCP and AI later.

## Roadmap

| Phase | Deliverable |
| --- | --- |
| 0 | Workspace, shell, start screen (this release) |
| 1 | Kadee AI: providers and models, Ask AI, an AI pane per tab, tools as commands, MCP server |
| 2 | Kadee Markdown — port of mdcraft to a native editor |
| 3 | Collaboration: Yrs CRDT sync server (Axum, PostgreSQL, NATS, S3), Docker Compose and Helm chart, OIDC |
| 4 | Kadee PDF |
| 5 | Kadee Convert |
| 6 | Kadee Dashboards and Reports (DataFusion) |
| 7 | Crafting Apps (ArtCraft and SolidCraft): launch, file hand-off, embedded engines |

The full architecture and implementation plan is in the KadeeStudio plan document.

## License

MIT OR Apache-2.0. The Crafting App names belong to the ArtCraft team and Tangency Labs; KadeeStudio only lists them.
