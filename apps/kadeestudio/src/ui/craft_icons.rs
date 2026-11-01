//! The Crafting Apps' own icons (128 px PNGs from the ArtCraft launcher,
//! copied unmodified; see `assets/craft-icons/ATTRIBUTION.md`).

use egui::{Align2, Color32, CornerRadius, FontId, Rect, TextureHandle, TextureOptions, Ui, pos2};
use kadee_core::CraftApp;
use kadee_ui::Palette;
use std::collections::HashMap;

const ICONS: [(&str, &[u8]); 15] = [
    ("cadcraft", include_bytes!("../../assets/craft-icons/cadcraft.png")),
    ("deckcraft", include_bytes!("../../assets/craft-icons/deckcraft.png")),
    ("designcraft", include_bytes!("../../assets/craft-icons/designcraft.png")),
    ("effectcraft", include_bytes!("../../assets/craft-icons/effectcraft.png")),
    ("figcraft", include_bytes!("../../assets/craft-icons/figcraft.png")),
    ("filmcraft", include_bytes!("../../assets/craft-icons/filmcraft.png")),
    ("gridcraft", include_bytes!("../../assets/craft-icons/gridcraft.png")),
    ("lightcraft", include_bytes!("../../assets/craft-icons/lightcraft.png")),
    ("pdfcraft", include_bytes!("../../assets/craft-icons/pdfcraft.png")),
    ("photocraft", include_bytes!("../../assets/craft-icons/photocraft.png")),
    ("solidcraft", include_bytes!("../../assets/craft-icons/solidcraft.png")),
    ("soundcraft", include_bytes!("../../assets/craft-icons/soundcraft.png")),
    ("texturecraft", include_bytes!("../../assets/craft-icons/texturecraft.png")),
    ("vectorcraft", include_bytes!("../../assets/craft-icons/vectorcraft.png")),
    ("wordcraft", include_bytes!("../../assets/craft-icons/wordcraft.png")),
];

/// Icons turned into textures on first use.
#[derive(Default)]
pub struct CraftIcons {
    loaded: HashMap<&'static str, Option<TextureHandle>>,
}

impl CraftIcons {
    fn get(&mut self, ctx: &egui::Context, id: &'static str) -> Option<TextureHandle> {
        self.loaded
            .entry(id)
            .or_insert_with(|| {
                let bytes = ICONS.iter().find(|(i, _)| *i == id)?.1;
                let icon = eframe::icon_data::from_png_bytes(bytes).ok()?;
                let image = egui::ColorImage::from_rgba_unmultiplied([icon.width as usize, icon.height as usize], &icon.rgba);
                Some(ctx.load_texture(format!("craft-icon-{id}"), image, TextureOptions::LINEAR))
            })
            .clone()
    }

    /// Paint `app`'s icon in `rect`; its initial on a tile if there is no icon.
    pub fn paint(&mut self, ui: &Ui, rect: Rect, app: &CraftApp, dim: bool) {
        let tint = if dim { Color32::from_white_alpha(110) } else { Color32::WHITE };
        match self.get(ui.ctx(), app.id) {
            Some(tex) => {
                ui.painter().image(tex.id(), rect, Rect::from_min_max(pos2(0.0, 0.0), pos2(1.0, 1.0)), tint);
            }
            None => {
                let pal = Palette::of(ui);
                ui.painter().rect_filled(rect, CornerRadius::same((rect.height() * 0.24) as u8), pal.surface_hover);
                ui.painter().text(
                    rect.center(),
                    Align2::CENTER_CENTER,
                    &app.name[..1],
                    FontId::proportional(rect.height() * 0.45),
                    pal.text_muted,
                );
            }
        }
    }
}

#[cfg(test)]
mod tests {
    #[test]
    fn every_crafting_app_has_an_icon() {
        for app in kadee_core::CRAFT_APPS {
            let (_, bytes) = super::ICONS.iter().find(|(id, _)| *id == app.id).expect(app.name);
            let icon = eframe::icon_data::from_png_bytes(bytes).unwrap();
            assert_eq!((icon.width, icon.height), (128, 128), "{}", app.name);
        }
    }
}
