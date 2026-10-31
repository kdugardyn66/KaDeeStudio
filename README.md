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

A screenshot of the first frame for docs or CI:

```sh
EFRAME_SCREENSHOT_TO=home.png cargo run -p kadeestudio --features screenshot
```

## Build installers

`./build.sh` picks the script for the OS you are on; everything lands in `dist/`.

### macOS (`scripts/build-macos.sh`)

Needs Rust and the Xcode command line tools (`xcode-select --install`).

```sh
./scripts/build-macos.sh               # this Mac's architecture → dist/KadeeStudio.app + .dmg
./scripts/build-macos.sh --run         # build the .app and open it
./scripts/build-macos.sh --universal   # Apple silicon + Intel in one binary
./scripts/build-macos.sh --no-dmg      # .app only
```

- The .app gets an Info.plist (bundle id `be.kadee.KadeeStudio`, macOS 11 or newer) and the icon from `assets/app-icon`.
- The .dmg is made with `diskutil image` on macOS 26 and newer, where hdiutil is deprecated, and with hdiutil on older systems (the same flow as mdcraft). Finder lays out the window with the background from `packaging/macos`; allow Terminal to control Finder when macOS asks, or use `DMG_LAYOUT=0` for a plain .dmg.
- Without a certificate the app is ad-hoc signed: it runs on the Mac that built it. To distribute, sign and notarize:

```sh
xcrun notarytool store-credentials kadee-notary --apple-id you@example.com --team-id TEAMID
APPLE_SIGNING_IDENTITY="Developer ID Application: Your Name (TEAMID)" \
APPLE_NOTARY_PROFILE=kadee-notary ./scripts/build-macos.sh --universal
```

### Linux (`scripts/build-linux.sh`)

```sh
sudo apt install build-essential pkg-config libxkbcommon-dev libwayland-dev libx11-dev \
  libxcursor-dev libxrandr-dev libxi-dev libgl1-mesa-dev libgtk-3-dev     # Debian / Ubuntu
./scripts/build-linux.sh               # .tar.gz with install.sh, plus .deb / .rpm / AppImage
```

A .deb is made when `dpkg-deb` is present, an .rpm when `rpmbuild` is, an AppImage when `appimagetool` is. Fedora packages are in the script's header.

### Windows (`scripts/build-windows.ps1`)

Needs Rust with the MSVC toolchain and Visual Studio Build Tools (C++ workload).

```powershell
.\scripts\build-windows.ps1              # x64 → portable .zip
.\scripts\build-windows.ps1 -Msi         # also an .msi (dotnet tool install --global wix)
.\scripts\build-windows.ps1 -Arch arm64
```

### Without your own machines

The **Release** workflow (`.github/workflows/release.yml`) builds the macOS .dmg (universal), the Linux .tar.gz, .deb, .rpm and AppImage, and the Windows .zip and .msi on GitHub's runners. Start it from Actions → Release → Run workflow, or push a tag such as `v0.1.0` to publish a GitHub release.

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
scripts/                 build-macos.sh, build-linux.sh, build-windows.ps1
packaging/               macOS dmg background, Linux .desktop + AppStream, Windows MSI (WiX)
assets/app-icon/         icon as PNG, .icns and .ico
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
