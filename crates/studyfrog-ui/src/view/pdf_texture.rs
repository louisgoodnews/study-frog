//! Page pictures as textures: uploaded on first use and let go when the state drops the picture.

use eframe::egui::{self, ColorImage, TextureHandle, TextureOptions};
use studyfrog_pdf::render::Pixels;

use crate::msg::Msg;
use crate::pdf_types::{PageKey, PdfMsg, PdfView};

/// Keyed by the pixels, not the requested scale: a fallback picture of another size (a thumbnail
/// shown while the sharp render is in flight) then gets its own handle and never takes over the
/// handle of the sharp picture.
fn texture_id(path: &str, page: usize, px: &Pixels) -> egui::Id {
    egui::Id::NULL.with(("pdf-texture", path, page, px.width, px.height))
}

/// The texture of a picture, uploaded on first use.
pub fn texture(ctx: &egui::Context, path: &str, key: PageKey, px: &Pixels) -> TextureHandle {
    let id = texture_id(path, key.page, px);
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

/// After drawing the picture of `key`: mark it as used again when it has fallen into the older
/// half of the cache, so the least recently used pictures are the ones let go.
pub fn mark_used(view: &PdfView, key: PageKey, out: &mut Vec<Msg>) {
    if crate::pdf_cache::stale(view, key) {
        out.push(Msg::Pdf(PdfMsg::Used(vec![key])));
    }
}

/// Let go of textures whose picture the state no longer holds.
pub fn prune(ctx: &egui::Context, view: &PdfView) {
    let live: Vec<egui::Id> = view
        .pictures
        .iter()
        .map(|(k, px)| texture_id(&view.path, k.page, px))
        .collect();
    let list_id = egui::Id::NULL.with("pdf-texture-list");
    ctx.data_mut(|d| {
        let list: Vec<egui::Id> = d.get_temp(list_id).unwrap_or_default();
        let (keep, drop): (Vec<_>, Vec<_>) = list.into_iter().partition(|id| live.contains(id));
        drop.iter().for_each(|id| d.remove::<TextureHandle>(*id));
        d.insert_temp(list_id, keep);
    });
}

#[cfg(test)]
mod tests {
    use super::texture_id;
    use studyfrog_pdf::render::Pixels;

    fn px(width: u32, height: u32) -> Pixels {
        Pixels {
            width,
            height,
            rgba: Vec::new(),
        }
    }

    #[test]
    fn a_fallback_of_another_size_has_its_own_handle() {
        let thumb = texture_id("a.pdf", 0, &px(250, 330));
        let sharp = texture_id("a.pdf", 0, &px(1250, 1650));
        assert_ne!(thumb, sharp);
    }

    #[test]
    fn the_same_pixels_keep_the_same_handle() {
        assert_eq!(
            texture_id("a.pdf", 3, &px(800, 1000)),
            texture_id("a.pdf", 3, &px(800, 1000))
        );
    }
}
