//! Links on a page: where they are and what they lead to. Nothing is followed here; the caller
//! decides, and a link to the web is only ever opened after the user agrees.

use lopdf::{Dictionary, Document, Object, ObjectId};

use crate::PdfBytes;
use crate::doc::text_string;
use crate::error::PdfError;
use crate::limits::{check_size, guarded};
use crate::text::page_transform;

/// Where a link leads.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum LinkTarget {
    /// An address (web, mail ...), exactly as the file gives it.
    Uri(String),
    /// A page of the same file (0-based).
    Page(usize),
}

/// A clickable area of a page, in points from the top left of the page as shown.
#[derive(Debug, Clone, PartialEq)]
pub struct PageLink {
    /// Left edge.
    pub x: f32,
    /// Top edge.
    pub y: f32,
    /// Width.
    pub width: f32,
    /// Height.
    pub height: f32,
    /// Where it leads.
    pub target: LinkTarget,
}

/// Most links read from one page.
pub const MAX_LINKS: usize = 500;
/// Deepest name tree followed when looking for a named destination.
const MAX_TREE_DEPTH: usize = 6;

fn bad(e: impl std::fmt::Display) -> PdfError {
    PdfError::Invalid(e.to_string())
}

fn deref<'a>(doc: &'a Document, o: &'a Object) -> Option<&'a Object> {
    doc.dereference(o).ok().map(|(_, o)| o)
}

fn dict_of<'a>(doc: &'a Document, o: &'a Object) -> Option<&'a Dictionary> {
    deref(doc, o)?.as_dict().ok()
}

fn name_text(o: &Object) -> Option<String> {
    match o {
        Object::Name(n) => Some(String::from_utf8_lossy(n).into_owned()),
        Object::String(s, _) => Some(text_string(s)),
        _ => None,
    }
}

/// The destination array `[page /Fit ...]` of a named destination, searched in the catalog's `Dests`
/// dictionary and in the `Names` tree.
fn named<'a>(doc: &'a Document, name: &str) -> Option<&'a Object> {
    let catalog = doc.catalog().ok()?;
    let old = catalog.get(b"Dests").ok().and_then(|o| dict_of(doc, o));
    if let Some(Ok(found)) = old.map(|d| d.get(name.as_bytes())) {
        return Some(found);
    }
    let root = catalog.get(b"Names").ok().and_then(|o| dict_of(doc, o))?;
    let tree = root.get(b"Dests").ok().and_then(|o| dict_of(doc, o))?;
    in_tree(doc, tree, name, 0)
}

fn in_tree<'a>(
    doc: &'a Document,
    node: &'a Dictionary,
    name: &str,
    depth: usize,
) -> Option<&'a Object> {
    if depth > MAX_TREE_DEPTH {
        return None;
    }
    if let Some(names) = node
        .get(b"Names")
        .ok()
        .and_then(|o| deref(doc, o))
        .and_then(|o| o.as_array().ok())
    {
        let at = names
            .chunks(2)
            .find(|p| p.first().and_then(name_text).as_deref() == Some(name))?;
        return at.get(1);
    }
    let kids = node
        .get(b"Kids")
        .ok()
        .and_then(|o| deref(doc, o))
        .and_then(|o| o.as_array().ok())?;
    kids.iter()
        .filter_map(|k| dict_of(doc, k))
        .find_map(|k| in_tree(doc, k, name, depth + 1))
}

/// The page a destination leads to (0-based).
fn dest_page(doc: &Document, dest: &Object, order: &[ObjectId]) -> Option<usize> {
    let dest = deref(doc, dest)?;
    let resolved = match dest {
        Object::Name(_) | Object::String(..) => deref(doc, named(doc, &name_text(dest)?)?)?,
        other => other,
    };
    let array = match resolved {
        Object::Dictionary(d) => deref(doc, d.get(b"D").ok()?)?.as_array().ok()?,
        other => other.as_array().ok()?,
    };
    let target = array.first()?.as_reference().ok()?;
    order.iter().position(|id| *id == target)
}

fn target_of(doc: &Document, annot: &Dictionary, order: &[ObjectId]) -> Option<LinkTarget> {
    if let Some(action) = annot.get(b"A").ok().and_then(|o| dict_of(doc, o)) {
        let kind = action.get(b"S").ok().and_then(name_text)?;
        return match kind.as_str() {
            "URI" => match action.get(b"URI").ok().and_then(|o| deref(doc, o))? {
                Object::String(s, _) => Some(LinkTarget::Uri(text_string(s))),
                _ => None,
            },
            "GoTo" => dest_page(doc, action.get(b"D").ok()?, order).map(LinkTarget::Page),
            _ => None,
        };
    }
    annot
        .get(b"Dest")
        .ok()
        .and_then(|d| dest_page(doc, d, order))
        .map(LinkTarget::Page)
}

fn rect_of(doc: &Document, annot: &Dictionary, m: [f64; 6]) -> Option<(f32, f32, f32, f32)> {
    let numbers: Vec<f32> = deref(doc, annot.get(b"Rect").ok()?)?
        .as_array()
        .ok()?
        .iter()
        .filter_map(|o| deref(doc, o).and_then(|o| o.as_float().ok()))
        .collect();
    let [x1, y1, x2, y2] = numbers.get(..4)?.try_into().ok()?;
    let map = |px: f32, py: f32| {
        let (px, py) = (f64::from(px), f64::from(py));
        (m[0] * px + m[2] * py + m[4], m[1] * px + m[3] * py + m[5])
    };
    let (a, b) = (map(x1, y1), map(x2, y2));
    let (left, right) = (a.0.min(b.0) as f32, a.0.max(b.0) as f32);
    let (top, bottom) = (a.1.min(b.1) as f32, a.1.max(b.1) as f32);
    Some((left, top, right - left, bottom - top))
}

/// The links on one page (0-based) that lead somewhere this program understands: an address or
/// another page of the file. Other kinds of action (launch, JavaScript, forms) are ignored, never run.
pub fn pdf_page_links(bytes: &PdfBytes, page: usize) -> Result<Vec<PageLink>, PdfError> {
    check_size(bytes)?;
    let bytes = bytes.clone();
    guarded(move || read_links(&bytes, page))
}

/// [`pdf_page_links`] without its own thread, for callers that already run guarded.
pub(crate) fn read_links(bytes: &PdfBytes, page: usize) -> Result<Vec<PageLink>, PdfError> {
    {
        let doc = Document::load_mem(bytes).map_err(bad)?;
        let order: Vec<ObjectId> = doc.get_pages().into_values().collect();
        let id = *order.get(page).ok_or(PdfError::NoSuchPage(page))?;
        let m = page_transform(bytes, page)?;
        let page_dict = doc.get_dictionary(id).map_err(bad)?;
        let Some(annots) = page_dict
            .get(b"Annots")
            .ok()
            .and_then(|o| deref(&doc, o))
            .and_then(|o| o.as_array().ok())
        else {
            return Ok(Vec::new());
        };
        let mut links = Vec::new();
        for annot in annots
            .iter()
            .filter_map(|a| dict_of(&doc, a))
            .take(MAX_LINKS * 4)
        {
            if annot.get(b"Subtype").ok().and_then(name_text).as_deref() != Some("Link") {
                continue;
            }
            let (Some(target), Some((x, y, width, height))) =
                (target_of(&doc, annot, &order), rect_of(&doc, annot, m))
            else {
                continue;
            };
            links.push(PageLink {
                x,
                y,
                width,
                height,
                target,
            });
            if links.len() >= MAX_LINKS {
                break;
            }
        }
        Ok(links)
    }
}
