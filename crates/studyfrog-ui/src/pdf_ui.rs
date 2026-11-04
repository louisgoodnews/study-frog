//! The PDF viewer: opening a file, turning pages, zooming, searching, selecting. Pure.

use crate::effect::Effect;
use crate::pdf_load;
use crate::pdf_types::{PdfEffect, PdfMsg, PdfView, Side, ZOOM_STEPS, Zoom};
use crate::state::{Notice, NoticeKind, Screen, State};

fn stem(path: &str) -> String {
    let name = path.rsplit('/').next().unwrap_or(path);
    name.rsplit_once('.').map_or(name, |(s, _)| s).to_owned()
}

pub(crate) fn with(state: State, view: PdfView) -> State {
    State {
        pdf: Some(view),
        ..state
    }
}

pub(crate) fn fail(state: State, text: String) -> (State, Vec<Effect>) {
    let notice = Some(Notice {
        kind: NoticeKind::Error,
        text,
    });
    (State { notice, ..state }, Vec::new())
}

/// Open a PDF file in the viewer.
pub fn open(state: State, path: String, page: usize) -> (State, Vec<Effect>) {
    let (state, mut effects) = crate::nav::navigate(state, Screen::Pdf);
    if state.screen != Screen::Pdf {
        return (state, effects);
    }
    let view = PdfView {
        title: stem(&path),
        path: path.clone(),
        page,
        color: "yellow".to_owned(),
        ..PdfView::default()
    };
    effects.push(Effect::Pdf(PdfEffect::Open(path)));
    (with(state, view), effects)
}

fn opened(
    state: State,
    path: String,
    result: Result<Box<studyfrog_pdf::doc::PdfInfo>, String>,
) -> (State, Vec<Effect>) {
    let Some(view) = state.pdf.clone().filter(|v| v.path == path) else {
        return (state, Vec::new());
    };
    match result {
        Ok(info) => {
            let side = if info.outline.is_empty() {
                Side::Pages
            } else {
                Side::Outline
            };
            let title = info
                .title
                .clone()
                .filter(|t| !t.trim().is_empty())
                .unwrap_or(view.title.clone());
            let page = view.page.min(info.page_count.saturating_sub(1));
            let view_path = path.clone();
            let marks = Effect::Pdf(PdfEffect::LoadMarks(path.clone()));
            let sourced = crate::pdf_extract_ui::ask_sourced(&path);
            let ask = Effect::Pdf(PdfEffect::Text { path, page });
            (
                with(
                    state,
                    PdfView {
                        info: Some(*info),
                        title,
                        side,
                        page,
                        ..view
                    },
                ),
                vec![
                    ask,
                    marks,
                    sourced,
                    Effect::Pdf(PdfEffect::LoadScanned(view_path)),
                ],
            )
        }
        Err(e) => (
            with(
                state,
                PdfView {
                    error: Some(e),
                    ..view
                },
            ),
            Vec::new(),
        ),
    }
}

fn page_count(view: &PdfView) -> usize {
    view.info.as_ref().map_or(0, |i| i.page_count)
}

pub(crate) fn go_to(state: State, page: usize) -> (State, Vec<Effect>) {
    let Some(view) = state.pdf.clone() else {
        return (state, Vec::new());
    };
    let last = page_count(&view).saturating_sub(1);
    let page = page.min(last);
    if page_count(&view) == 0 || page == view.page && view.text.is_some() {
        return (state, Vec::new());
    }
    let ask = Effect::Pdf(PdfEffect::Text {
        path: view.path.clone(),
        page,
    });
    (
        with(
            state,
            PdfView {
                page,
                text: None,
                selection: Vec::new(),
                ..view
            },
        ),
        vec![ask],
    )
}

fn zoom_step(view: &PdfView, up: bool) -> Zoom {
    let now = match view.zoom {
        Zoom::Percent(p) => p,
        Zoom::FitWidth | Zoom::FitPage => 100,
    };
    let next = if up {
        ZOOM_STEPS.iter().copied().find(|s| *s > now)
    } else {
        ZOOM_STEPS.iter().rev().copied().find(|s| *s < now)
    };
    Zoom::Percent(next.unwrap_or(now))
}

/// Handle the viewer's messages.
pub fn update(state: State, msg: PdfMsg) -> (State, Vec<Effect>) {
    let page = state.pdf.as_ref().map_or(0, |v| v.page);
    match msg {
        PdfMsg::Open(path, p) => open(state, path, p),
        PdfMsg::Opened(path, result) => opened(state, path, result),
        PdfMsg::Close => crate::nav::navigate(State { pdf: None, ..state }, Screen::Library),
        PdfMsg::GoTo(p) => go_to(state, p),
        PdfMsg::Next => go_to(state, page + 1),
        PdfMsg::Previous => go_to(state, page.saturating_sub(1)),
        PdfMsg::ZoomIn | PdfMsg::ZoomOut => {
            let up = msg == PdfMsg::ZoomIn;
            let view = state.pdf.clone().map(|v| PdfView {
                zoom: zoom_step(&v, up),
                ..v
            });
            (State { pdf: view, ..state }, Vec::new())
        }
        PdfMsg::SetZoom(z) => (
            State {
                pdf: state.pdf.clone().map(|v| PdfView { zoom: z, ..v }),
                ..state
            },
            Vec::new(),
        ),
        PdfMsg::Need(key) => pdf_load::need(state, key),
        PdfMsg::Rendered(path, key, result) => pdf_load::rendered(state, path, key, result),
        PdfMsg::TextLoaded(path, p, result) => pdf_load::text_loaded(state, path, p, result),
        PdfMsg::Query(text) => pdf_load::query(state, text),
        PdfMsg::Found(path, q, result) => pdf_load::found(state, path, q, result),
        PdfMsg::NextHit => pdf_load::hit(state, true),
        PdfMsg::PreviousHit => pdf_load::hit(state, false),
        PdfMsg::Select(words) => (
            State {
                pdf: state.pdf.clone().map(|v| PdfView {
                    selection: words,
                    ..v
                }),
                ..state
            },
            Vec::new(),
        ),
        PdfMsg::Scanned(path, r) => scanned(state, path, r),
        PdfMsg::Link(m) => crate::pdf_link_ui::update(state, m),
        PdfMsg::Library(m) => crate::pdf_library_ui::update(state, m),
        PdfMsg::Draft(m) => crate::pdf_draft_ui::update(state, m),
        PdfMsg::Extract(m) => crate::pdf_extract_ui::update(state, m),
        PdfMsg::Arrange(m) => crate::pdf_arrange_ui::update(state, m),
        PdfMsg::Marks(m) => crate::pdf_marks_ui::update(state, m),
        PdfMsg::Export(m) => crate::pdf_export_ui::update(state, m),
        PdfMsg::ShowSide(side) => (
            State {
                pdf: state.pdf.clone().map(|v| PdfView { side, ..v }),
                ..state
            },
            Vec::new(),
        ),
    }
}

fn scanned(state: State, path: String, result: Result<Vec<usize>, String>) -> (State, Vec<Effect>) {
    let Some(view) = state.pdf.clone().filter(|v| v.path == path) else {
        return (state, Vec::new());
    };
    // An unreadable index only means no note about scans is shown.
    let scanned = result.unwrap_or_default();
    (with(state, PdfView { scanned, ..view }), Vec::new())
}
