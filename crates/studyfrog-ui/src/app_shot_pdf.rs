//! The PDF viewer for `--screenshot` (BUG-0010): `pdf:<vault path>[@<zoom>][+pages]` opens the
//! file at its first page; `<zoom>` is `fit-width`, `fit-page` or a percentage (`100` is the
//! original size), `+pages` opens the thumbnail list, `+outline` the outline.

use crate::effect::Effect;
use crate::model::{Model, apply};
use crate::msg::Msg;
use crate::pdf_types::{PdfMsg, Side, Zoom};

/// What a `pdf:` screen name asks for.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PdfShot {
    /// The vault-relative path of the file.
    pub path: String,
    /// How big the page is shown.
    pub zoom: Option<Zoom>,
    /// The side panel.
    pub side: Option<Side>,
}

fn zoom_named(word: &str) -> Option<Zoom> {
    match word {
        "fit-width" | "width" => Some(Zoom::FitWidth),
        "fit-page" | "page" | "fit-height" => Some(Zoom::FitPage),
        other => other
            .trim_end_matches('%')
            .parse()
            .ok()
            .map(|p: u32| Zoom::Percent(p.clamp(10, 800))),
    }
}

/// Read a `pdf:` screen name.
pub fn parse(name: &str) -> Option<PdfShot> {
    let rest = name.strip_prefix("pdf:")?;
    let mut parts = rest.split('+');
    let head = parts.next()?;
    let side = parts.fold(None, |side, flag| match flag {
        "pages" => Some(Side::Pages),
        "outline" => Some(Side::Outline),
        _ => side,
    });
    let (path, zoom) = match head.rsplit_once('@') {
        Some((p, z)) => (p, zoom_named(z)),
        None => (head, None),
    };
    (!path.is_empty()).then(|| PdfShot {
        path: path.to_owned(),
        zoom,
        side,
    })
}

/// Most frames the picture waits for the worker's renders (a debug build renders slowly).
const MOST_WAIT: u32 = 1200;

/// Whether the PDF viewer still waits for pictures, so the screenshot should wait too;
/// `waited` counts the frames already held back.
pub fn waiting(model: &Model, waited: u32) -> bool {
    let pending = model
        .state
        .pdf
        .as_ref()
        .is_some_and(|v| !v.requested.is_empty());
    pending && waited < MOST_WAIT
}

/// Open the viewer as `name` asks, if it is a `pdf:` name.
pub fn setup(model: &mut Model, name: &str) -> Option<Vec<Effect>> {
    let shot = parse(name)?;
    let mut fx = apply(model, Msg::Pdf(PdfMsg::Open(shot.path, 0)));
    if let Some(z) = shot.zoom {
        fx.extend(apply(model, Msg::Pdf(PdfMsg::SetZoom(z))));
    }
    if let Some(s) = shot.side {
        fx.extend(apply(model, Msg::Pdf(PdfMsg::ShowSide(s))));
    }
    Some(fx)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn names_carry_the_path_the_zoom_and_the_side_panel() {
        let shot = parse("pdf:bio/small.pdf@fit-width+pages").unwrap();
        assert_eq!(shot.path, "bio/small.pdf");
        assert_eq!(shot.zoom, Some(Zoom::FitWidth));
        assert_eq!(shot.side, Some(Side::Pages));
        let plain = parse("pdf:a b/c.pdf").unwrap();
        assert_eq!((plain.zoom, plain.side), (None, None));
        assert_eq!(
            parse("pdf:x.pdf@150%").unwrap().zoom,
            Some(Zoom::Percent(150))
        );
        assert_eq!(
            parse("pdf:x.pdf@fit-page").unwrap().zoom,
            Some(Zoom::FitPage)
        );
        assert!(parse("pdf:").is_none());
        assert!(parse("library").is_none());
    }
}
