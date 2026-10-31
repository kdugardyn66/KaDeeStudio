//! Catalogue of the ArtCraft Crafting Apps that KadeeStudio will host.
//!
//! Phase 0 lists them on the start screen. Phase 6 adds launching,
//! file hand-off and embedding of their engine crates.

/// One Crafting App.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct CraftApp {
    pub name: &'static str,
    pub purpose: &'static str,
}

/// Where the ArtCraft team publishes the launcher for all Crafting Apps.
pub const LAUNCHER_REPO: &str = "https://github.com/storytold/craft-launcher";

/// The Crafting Apps known today.
pub const CRAFT_APPS: [CraftApp; 10] = [
    CraftApp { name: "WordCraft", purpose: "Word processing" },
    CraftApp { name: "GridCraft", purpose: "Spreadsheets and data" },
    CraftApp { name: "DeckCraft", purpose: "Presentations" },
    CraftApp { name: "PdfCraft", purpose: "Organise and protect PDFs" },
    CraftApp { name: "DesignCraft", purpose: "Page layout and publishing" },
    CraftApp { name: "PhotoCraft", purpose: "Image editing, PSD files" },
    CraftApp { name: "VectorCraft", purpose: "Vector illustration" },
    CraftApp { name: "LightCraft", purpose: "Photo library and raw" },
    CraftApp { name: "FilmCraft", purpose: "Video editing" },
    CraftApp { name: "EffectCraft", purpose: "Motion graphics" },
];
