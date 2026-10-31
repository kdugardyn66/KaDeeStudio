//! Catalogue of the Crafting Apps that KadeeStudio will host: the ten
//! ArtCraft apps plus SolidCraft (3D CAD, from Tangency Labs).
//!
//! Phase 0 lists them on the start screen and shows their files under
//! **All**. Phase 7 ([`CRAFT_PHASE`]) adds launching, file hand-off and embedding.

use crate::doctype::DocGroup;

/// One Crafting App and the files it opens.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct CraftApp {
    pub name: &'static str,
    pub purpose: &'static str,
    /// Who publishes it.
    pub maker: &'static str,
    /// Short badge text for its files.
    pub badge: &'static str,
    /// Filter pill its files appear under.
    pub group: DocGroup,
    /// Lower-case extensions without the dot.
    pub extensions: &'static [&'static str],
}

/// Where the ArtCraft team publishes the launcher for its apps.
pub const LAUNCHER_REPO: &str = "https://github.com/storytold/craft-launcher";
/// SolidCraft's repository.
pub const SOLIDCRAFT_REPO: &str = "https://github.com/tangency-labs/solidcraft";

const ARTCRAFT: &str = "ArtCraft";

/// Roadmap phase that integrates the Crafting Apps.
pub const CRAFT_PHASE: u8 = 7;

/// The Crafting Apps known today.
///
/// `designcraft` and `effectcraft` follow VectorCraft's `.vectorcraft`
/// project-file pattern and still need confirming against those apps.
pub const CRAFT_APPS: [CraftApp; 11] = [
    CraftApp {
        name: "WordCraft",
        purpose: "Word processing",
        maker: ARTCRAFT,
        badge: "DOC",
        group: DocGroup::Office,
        extensions: &["docx"],
    },
    CraftApp {
        name: "GridCraft",
        purpose: "Spreadsheets and data",
        maker: ARTCRAFT,
        badge: "XLS",
        group: DocGroup::Office,
        extensions: &["xlsx", "csv"],
    },
    CraftApp { name: "DeckCraft", purpose: "Presentations", maker: ARTCRAFT, badge: "PPT", group: DocGroup::Office, extensions: &["pptx"] },
    // PDF opens in Kadee PDF first; PdfCraft is the alternative.
    CraftApp {
        name: "PdfCraft",
        purpose: "Organise and protect PDFs",
        maker: ARTCRAFT,
        badge: "PDF",
        group: DocGroup::Pdf,
        extensions: &["pdf"],
    },
    CraftApp {
        name: "DesignCraft",
        purpose: "Page layout and publishing",
        maker: ARTCRAFT,
        badge: "DTP",
        group: DocGroup::Graphics,
        extensions: &["designcraft"],
    },
    CraftApp {
        name: "PhotoCraft",
        purpose: "Image editing, PSD files",
        maker: ARTCRAFT,
        badge: "IMG",
        group: DocGroup::Graphics,
        extensions: &["psd", "png", "jpg", "jpeg", "webp", "tif", "tiff"],
    },
    CraftApp {
        name: "VectorCraft",
        purpose: "Vector illustration",
        maker: ARTCRAFT,
        badge: "VEC",
        group: DocGroup::Graphics,
        extensions: &["svg", "eps", "dxf", "emf", "wmf", "vectorcraft"],
    },
    CraftApp {
        name: "LightCraft",
        purpose: "Photo library and raw",
        maker: ARTCRAFT,
        badge: "RAW",
        group: DocGroup::Media,
        extensions: &["dng", "cr2", "cr3", "nef", "arw", "raf", "orf", "rw2"],
    },
    CraftApp {
        name: "FilmCraft",
        purpose: "Video editing",
        maker: ARTCRAFT,
        badge: "VID",
        group: DocGroup::Media,
        extensions: &["mp4", "mov", "mkv", "webm"],
    },
    CraftApp {
        name: "EffectCraft",
        purpose: "Motion graphics",
        maker: ARTCRAFT,
        badge: "FX",
        group: DocGroup::Media,
        extensions: &["effectcraft"],
    },
    CraftApp {
        name: "SolidCraft",
        purpose: "3D CAD and modelling",
        maker: "Tangency Labs",
        badge: "3D",
        group: DocGroup::Cad,
        extensions: &["step", "stp", "iges", "igs", "3mf", "stl", "obj", "gltf", "glb"],
    },
];

/// The Crafting App that opens files with this extension (lower-case, no dot).
pub fn craft_for_extension(ext: &str) -> Option<&'static CraftApp> {
    CRAFT_APPS.iter().find(|a| a.extensions.contains(&ext))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn includes_solidcraft_and_its_cad_formats() {
        let s = craft_for_extension("step").unwrap();
        assert_eq!(s.name, "SolidCraft");
        assert_eq!(s.group, DocGroup::Cad);
        assert_eq!(craft_for_extension("stl").unwrap().name, "SolidCraft");
    }

    #[test]
    fn no_extension_is_claimed_twice() {
        let mut seen = std::collections::HashSet::new();
        for a in CRAFT_APPS {
            for e in a.extensions {
                assert!(seen.insert(*e), "{e} is listed twice");
            }
        }
    }
}
