//! Pictures out of a PDF: the pictures on a page, or a rectangle of the page drawn as a picture,
//! saved into the vault's `assets/` folder under the usual picture rules.

use std::sync::Arc;

use studyfrog_pdf::error::describe_pdf_error;
use studyfrog_pdf::images::pdf_page_images;
use studyfrog_pdf::render::{PageRect, pdf_render_region, pixels_to_png};

use crate::images::store_image;
use crate::session::{ServiceError, VaultSession};

/// How sharp a snapshot is drawn: pixels per point.
pub const SNAPSHOT_SCALE: f32 = 2.0;

fn bytes_of(session: &VaultSession, rel: &str) -> Result<Arc<Vec<u8>>, ServiceError> {
    let clean = !rel.starts_with('/') && !rel.split('/').any(|p| p == ".." || p.is_empty());
    if !clean || !rel.to_lowercase().ends_with(".pdf") {
        return Err(ServiceError::Invalid(
            "only PDF files inside the vault can be used".to_owned(),
        ));
    }
    let data = std::fs::read(session.root.join(rel))
        .map_err(|e| ServiceError::Io(format!("cannot read {rel}: {e}")))?;
    Ok(Arc::new(data))
}

fn stem(rel: &str) -> String {
    let name = rel.rsplit('/').next().unwrap_or(rel);
    name.rsplit_once('.').map_or(name, |(s, _)| s).to_owned()
}

fn invalid(e: &studyfrog_pdf::error::PdfError) -> ServiceError {
    ServiceError::Invalid(describe_pdf_error(e))
}

/// Save the pictures of one page (0-based); returns their vault-relative paths, none if the page has no pictures.
pub fn save_page_pictures(
    session: &VaultSession,
    pdf: &str,
    page: usize,
) -> Result<Vec<String>, ServiceError> {
    let bytes = bytes_of(session, pdf)?;
    let images = pdf_page_images(&bytes, page).map_err(|e| invalid(&e))?;
    images
        .iter()
        .enumerate()
        .map(|(i, img)| {
            store_image(
                session,
                &format!("{}-p{}-{}", stem(pdf), page + 1, i + 1),
                "png",
                &img.png,
            )
        })
        .collect()
}

/// Draw a rectangle of a page (points from the top left) and save it as a picture.
pub fn save_page_region(
    session: &VaultSession,
    pdf: &str,
    page: usize,
    rect: PageRect,
) -> Result<String, ServiceError> {
    if !(rect.width >= 4.0 && rect.height >= 4.0) {
        return Err(ServiceError::Invalid(
            "drag a rectangle at least a few points wide and high".to_owned(),
        ));
    }
    let bytes = bytes_of(session, pdf)?;
    let pixels = pdf_render_region(&bytes, page, rect, SNAPSHOT_SCALE).map_err(|e| invalid(&e))?;
    let png = pixels_to_png(&pixels).map_err(|e| invalid(&e))?;
    store_image(
        session,
        &format!("{}-p{}-snapshot", stem(pdf), page + 1),
        "png",
        &png,
    )
}
