//! What a run shows for an image occlusion card (STORY-0109, `docs/specs/image-occlusion.md`): the picture
//! with its masks, one asked at a time (they take turns with the card's reviews), and its label as the answer.
//! Also making a new occlusion card from a picture.

use studyfrog_md::doc::{CardDoc, Doc};
use studyfrog_md::occlusion::{
    Mask, Occlusion, OcclusionMode, occlusion_of, with_occlusion, without_images,
};
use studyfrog_md::render::render_doc;

use crate::content::ItemContent;
use crate::create::{
    IdSource, meta, refresh, render_error, slug, unique_name, with_new_defaults, write_new,
};
use crate::create_more::Created;
use crate::session::{ServiceError, VaultInfo, VaultSession};

/// The picture and masks a run draws.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct OcclusionView {
    /// The picture, relative to the vault.
    pub image: String,
    /// Hide all or hide one.
    pub mode: OcclusionMode,
    /// Every mask.
    pub masks: Vec<Mask>,
    /// The mask asked now.
    pub active: usize,
}

/// What the prompt says when the card's front has nothing but the picture.
pub const DEFAULT_PROMPT: &str = "What is hidden under the marked box?";

/// `content` turned into its occlusion form after `reviews` reviews; unchanged for other items or a
/// card without masks.
pub fn with_occlusion_view(doc: &Doc, content: ItemContent, reviews: usize) -> ItemContent {
    let (Some(o), Doc::Card(card)) = (occlusion_of(doc), doc) else {
        return content;
    };
    if o.masks.is_empty() {
        return content;
    }
    let active = reviews % o.masks.len();
    let label = o.masks[active].label.clone();
    let prompt = match without_images(&card.front) {
        p if p.is_empty() => DEFAULT_PROMPT.to_owned(),
        p => p,
    };
    let shown = if label.is_empty() {
        "(the uncovered part)".to_owned()
    } else {
        format!("**{label}**")
    };
    let answer = match card.back.trim() {
        "" => shown,
        more => format!("{shown}\n\n{more}"),
    };
    ItemContent {
        prompt,
        answer,
        expected: (!label.is_empty()).then(|| label.clone()),
        occlusion: Some(Box::new(OcclusionView {
            image: o.image,
            mode: o.mode,
            masks: o.masks,
            active,
        })),
        ..content
    }
}

/// The label of the mask asked after `reviews` reviews, for comparing a typed answer.
pub fn asked_label(doc: &Doc, reviews: usize) -> Option<String> {
    let o = occlusion_of(doc)?;
    let m = o.masks.get(reviews % o.masks.len().max(1))?;
    (!m.label.is_empty()).then(|| m.label.clone())
}

/// A new card in the stack `folder` showing the vault picture `image`, with no masks yet (they are drawn
/// on its page). Its front holds the picture so other Markdown readers show it too.
pub fn create_occlusion_card(
    session: &mut VaultSession,
    folder: &str,
    image: &str,
    ids: IdSource,
) -> Result<(VaultInfo, Created), ServiceError> {
    let dir = session.root.join(folder);
    if folder.is_empty() || !dir.join(studyfrog_store::stackmap::STACK_FILE).is_file() {
        return Err(ServiceError::Invalid(
            "choose the stack the card goes into".to_owned(),
        ));
    }
    let name = image.rsplit('/').next().unwrap_or(image);
    let stem = name.rsplit_once('.').map_or(name, |(s, _)| s);
    let title = format!("Occlusion: {stem}");
    let doc = Doc::Card(CardDoc {
        meta: meta(&title, &[], ids),
        reversible: false,
        front: format!("![{stem}]({image})\n\n{DEFAULT_PROMPT}"),
        back: String::new(),
        extra: Vec::new(),
    });
    let o = Occlusion {
        image: image.to_owned(),
        mode: OcclusionMode::HideAll,
        masks: Vec::new(),
    };
    let doc = with_new_defaults(with_occlusion(&doc, Some(&o)));
    let doc = crate::item_keys::with_new_key(&session.conn, doc)?;
    let uid = studyfrog_md::doc::doc_meta(&doc).uid;
    let file = unique_name(&slug(&title), ".md", |n| dir.join(n).exists());
    write_new(&dir.join(&file), &render_doc(&doc).map_err(render_error)?)?;
    let created = Created {
        title,
        path: format!("{folder}/{file}"),
        uid: Some(uid),
    };
    Ok((refresh(session)?, created))
}
