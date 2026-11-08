//! Drawing a picture of a Markdown text.

use eframe::egui::{self, Id, SizeHint, TextureOptions, load::TexturePoll};

use crate::images::image_uri;

fn root_key() -> Id {
    Id::NULL.with("studyfrog-vault-root")
}

/// Tell the picture drawing where the open vault is (once per frame).
pub fn set_vault_root(ctx: &egui::Context, root: Option<&str>) {
    ctx.data_mut(|d| match root {
        Some(r) => {
            d.insert_temp(root_key(), r.to_owned());
        }
        None => d.remove::<String>(root_key()),
    });
}

/// The picture at `path` (relative to the vault), or its description when it cannot be shown. egui's
/// own image widget prints a warning sign when loading fails, and the bundled fonts have no glyph for
/// it, so the loading is handled here.
pub fn draw_image(ui: &mut egui::Ui, alt: &str, path: &str) {
    let root = ui.ctx().data(|d| d.get_temp::<String>(root_key()));
    let name = if alt.is_empty() { path } else { alt };
    let Some(uri) = root.as_deref().and_then(|r| image_uri(r, path)) else {
        ui.weak(format!("[picture: {name}]"))
            .on_hover_text("Only pictures inside the vault are shown.");
        return;
    };
    match ui
        .ctx()
        .try_load_texture(&uri, TextureOptions::default(), SizeHint::default())
    {
        Ok(TexturePoll::Ready { texture }) => {
            let image = egui::Image::from_texture(texture)
                .max_width(ui.available_width())
                .alt_text(alt.to_owned());
            ui.add(image);
        }
        Ok(TexturePoll::Pending { .. }) => {
            ui.ctx().request_repaint();
            ui.weak(format!("[loading picture: {name}]"));
        }
        Err(_) => {
            ui.weak(format!("[picture: {name} could not be shown]"));
        }
    }
}

/// The texture of the vault picture at `path`: `Ok(None)` while it loads, `Err` with what to say when it
/// cannot be shown.
pub fn vault_texture(
    ui: &egui::Ui,
    path: &str,
) -> Result<Option<egui::load::SizedTexture>, String> {
    let root = ui.ctx().data(|d| d.get_temp::<String>(root_key()));
    let uri = root
        .as_deref()
        .and_then(|r| image_uri(r, path))
        .ok_or_else(|| format!("[picture: {path}] Only pictures inside the vault are shown."))?;
    match ui
        .ctx()
        .try_load_texture(&uri, TextureOptions::default(), SizeHint::default())
    {
        Ok(TexturePoll::Ready { texture }) => Ok(Some(texture)),
        Ok(TexturePoll::Pending { .. }) => {
            ui.ctx().request_repaint();
            Ok(None)
        }
        Err(_) => Err(format!("[picture: {path} could not be shown]")),
    }
}
