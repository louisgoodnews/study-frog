//! Page pictures as textures: uploaded on first use and let go when the state drops the picture.

use eframe::egui::{self, ColorImage, TextureHandle, TextureOptions};
use studyfrog_pdf::render::Pixels;

use crate::pdf_types::{PageKey, PdfView};

fn texture_id(path: &str, key: PageKey) -> egui::Id {
    egui::Id::NULL.with(("pdf-texture", path, key.page, key.scale_milli))
}

/// The texture of a picture, uploaded on first use.
pub fn texture(ctx: &egui::Context, path: &str, key: PageKey, px: &Pixels) -> TextureHandle {
    let id = texture_id(path, key);
    if let Some(t) = ctx.data(|d| d.get_temp::<TextureHandle>(id)) {
        return t;
    }
    let image =
        ColorImage::from_rgba_unmultiplied([px.width as usize, px.height as usize], &px.rgba);
    let handle = ctx.load_texture(
        format!("pdf-{}-{}", key.page, key.scale_milli),
        image,
        TextureOptions::LINEAR,
    );
    let list_id = egui::Id::NULL.with("pdf-texture-list");
    ctx.data_mut(|d| {
        d.insert_temp(id, handle.clone());
        let mut list: Vec<egui::Id> = d.get_temp(list_id).unwrap_or_default();
        list.push(id);
        d.insert_temp(list_id, list);
    });
    handle
}

/// Let go of textures whose picture the state no longer holds.
pub fn prune(ctx: &egui::Context, view: &PdfView) {
    let live: Vec<egui::Id> = view
        .pictures
        .iter()
        .map(|(k, _)| texture_id(&view.path, *k))
        .collect();
    let list_id = egui::Id::NULL.with("pdf-texture-list");
    ctx.data_mut(|d| {
        let list: Vec<egui::Id> = d.get_temp(list_id).unwrap_or_default();
        let (keep, drop): (Vec<_>, Vec<_>) = list.into_iter().partition(|id| live.contains(id));
        drop.iter().for_each(|id| d.remove::<TextureHandle>(*id));
        d.insert_temp(list_id, keep);
    });
}
