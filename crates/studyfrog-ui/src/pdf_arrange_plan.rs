//! Editing the plan of a new PDF: the pure steps behind the rearranging screen.

use studyfrog_pdf::edit::PageSpec;

use crate::pdf_arrange_types::Arrange;
use crate::pdf_types::PdfView;

pub(crate) fn default_name(path: &str) -> String {
    let stem = path
        .strip_suffix(".pdf")
        .or_else(|| path.strip_suffix(".PDF"))
        .unwrap_or(path);
    format!("{stem}-edited.pdf")
}

pub(crate) fn start(view: &PdfView) -> Arrange {
    let count = view.info.as_ref().map_or(0, |i| i.page_count);
    let info = view.info.as_ref();
    let field = |v: Option<&Option<String>>| v.and_then(Clone::clone).unwrap_or_default();
    Arrange {
        sources: vec![view.path.clone()],
        plan: (0..count)
            .map(|page| PageSpec {
                source: 0,
                page,
                rotate: 0,
            })
            .collect(),
        name: default_name(&view.path),
        include_marks: !view.marks.is_empty(),
        props: [
            field(info.map(|i| &i.title)),
            field(info.map(|i| &i.author)),
            field(info.map(|i| &i.subject)),
        ],
        selected: Some(view.page.min(count.saturating_sub(1))),
        ..Arrange::default()
    }
}

pub(crate) fn change(a: &Arrange, plan: Vec<PageSpec>, selected: Option<usize>) -> Arrange {
    let mut undo = a.undo.clone();
    undo.push(a.plan.clone());
    Arrange {
        plan,
        undo,
        selected,
        message: None,
        ..a.clone()
    }
}

pub(crate) fn rotate(a: &Arrange, by: i32) -> Option<Arrange> {
    let i = a.selected?;
    let mut plan = a.plan.clone();
    let step = plan.get_mut(i)?;
    step.rotate = (step.rotate + by).rem_euclid(360);
    Some(change(a, plan, Some(i)))
}

pub(crate) fn delete(a: &Arrange) -> Option<Arrange> {
    let i = a.selected?;
    if a.plan.len() <= 1 || i >= a.plan.len() {
        return None;
    }
    let mut plan = a.plan.clone();
    plan.remove(i);
    let last = plan_last(&plan);
    Some(change(a, plan, Some(i.min(last))))
}

pub(crate) fn plan_last(plan: &[PageSpec]) -> usize {
    plan.len().saturating_sub(1)
}

pub(crate) fn shift(a: &Arrange, by: i32) -> Option<Arrange> {
    let i = a.selected?;
    let j = i
        .checked_add_signed(by as isize)
        .filter(|j| *j < a.plan.len())?;
    let mut plan = a.plan.clone();
    plan.swap(i, j);
    Some(change(a, plan, Some(j)))
}

pub(crate) fn added(a: &Arrange, path: String, pages: usize) -> Arrange {
    let source = a.sources.len();
    let mut plan = a.plan.clone();
    plan.extend((0..pages).map(|page| PageSpec {
        source,
        page,
        rotate: 0,
    }));
    let mut sources = a.sources.clone();
    sources.push(path);
    Arrange {
        sources,
        ..change(a, plan, a.selected)
    }
}

pub(crate) fn start_keeping(a: &Arrange, view: &PdfView) -> Arrange {
    let fresh = start(view);
    change(a, fresh.plan, fresh.selected)
}
