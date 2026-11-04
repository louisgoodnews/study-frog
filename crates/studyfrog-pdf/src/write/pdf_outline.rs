//! PDF document outline (/Outlines) generation for navigation bookmarks.

use lopdf::{Document, Object, ObjectId, StringFormat, dictionary};

use super::primitives::HeadingOutline;
use super::to_unicode::encode_text_winansi;

struct OutlineNode {
    id: ObjectId,
    title: String,
    page_ref: ObjectId,
    top_pdf: f32,
    children: Vec<OutlineNode>,
}

fn build_outline_forest(
    headings: &[HeadingOutline],
    page_refs: &[ObjectId],
    page_heights: &[f32],
    doc: &mut Document,
) -> Vec<OutlineNode> {
    let mut stack: Vec<(u8, OutlineNode)> = Vec::new();
    let mut roots: Vec<OutlineNode> = Vec::new();

    for h in headings {
        if h.page_index >= page_refs.len() {
            continue;
        }
        let page_ref = page_refs[h.page_index];
        let h_page = page_heights.get(h.page_index).copied().unwrap_or(841.89);
        let top_pdf = (h_page - h.y).max(0.0);

        let node = OutlineNode {
            id: doc.new_object_id(),
            title: h.title.clone(),
            page_ref,
            top_pdf,
            children: Vec::new(),
        };

        while let Some((lvl, _)) = stack.last() {
            if *lvl >= h.level {
                if let Some((_, popped)) = stack.pop() {
                    if let Some((_, parent)) = stack.last_mut() {
                        parent.children.push(popped);
                    } else {
                        roots.push(popped);
                    }
                }
            } else {
                break;
            }
        }
        stack.push((h.level, node));
    }

    while let Some((_, popped)) = stack.pop() {
        if let Some((_, parent)) = stack.last_mut() {
            parent.children.push(popped);
        } else {
            roots.push(popped);
        }
    }
    roots
}

fn emit_nodes(
    nodes: &[OutlineNode],
    parent_id: ObjectId,
    doc: &mut Document,
) -> (ObjectId, ObjectId, i64) {
    let count = nodes.len() as i64;
    let first_id = nodes[0].id;
    let last_id = nodes[nodes.len() - 1].id;

    for (i, node) in nodes.iter().enumerate() {
        let mut dict = dictionary! {
            "Title" => Object::String(encode_text_winansi(&node.title), StringFormat::Literal),
            "Parent" => parent_id,
            "Dest" => vec![
                Object::Reference(node.page_ref),
                "XYZ".into(),
                54.into(),
                (node.top_pdf as i64).into(),
                Object::Null
            ],
        };
        if i > 0 {
            dict.set("Prev", nodes[i - 1].id);
        }
        if i + 1 < nodes.len() {
            dict.set("Next", nodes[i + 1].id);
        }
        if !node.children.is_empty() {
            let (child_first, child_last, child_count) = emit_nodes(&node.children, node.id, doc);
            dict.set("First", child_first);
            dict.set("Last", child_last);
            dict.set("Count", child_count);
        }
        doc.objects.insert(node.id, Object::Dictionary(dict));
    }

    (first_id, last_id, count)
}

/// Create the /Outlines object hierarchy and return the root /Outlines ObjectId.
pub fn build_pdf_outlines(
    headings: &[HeadingOutline],
    page_refs: &[ObjectId],
    page_heights: &[f32],
    doc: &mut Document,
) -> Option<ObjectId> {
    if headings.is_empty() || page_refs.is_empty() {
        return None;
    }

    let roots = build_outline_forest(headings, page_refs, page_heights, doc);
    if roots.is_empty() {
        return None;
    }

    let outlines_id = doc.new_object_id();
    let (first_id, last_id, count) = emit_nodes(&roots, outlines_id, doc);

    doc.objects.insert(
        outlines_id,
        Object::Dictionary(dictionary! {
            "Type" => "Outlines",
            "First" => first_id,
            "Last" => last_id,
            "Count" => count,
        }),
    );

    Some(outlines_id)
}
