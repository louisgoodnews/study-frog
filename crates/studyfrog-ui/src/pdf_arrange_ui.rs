//! Rearranging in the viewer: edit the plan, then write it as a new file. Pure.

use studyfrog_pdf::edit::{Properties, parse_page_ranges};
use studyfrog_service::pdf_ops::SaveRequest;

use crate::effect::Effect;
use crate::pdf_arrange_plan::{added, delete, rotate, shift, start, start_keeping};
use crate::pdf_arrange_types::{Arrange, ArrangeMsg};
use crate::pdf_types::{PdfEffect, PdfView};
use crate::pdf_ui::with;
use crate::state::{Notice, NoticeKind, State};

fn properties(a: &Arrange, view: &PdfView) -> Option<Properties> {
    let info = view.info.as_ref()?;
    let original =
        [&info.title, &info.author, &info.subject].map(|o| o.clone().unwrap_or_default());
    let pick = |i: usize| (a.props[i] != original[i]).then(|| a.props[i].clone());
    let props = Properties {
        title: pick(0),
        author: pick(1),
        subject: pick(2),
    };
    (props.title.is_some() || props.author.is_some() || props.subject.is_some()).then_some(props)
}

fn save(state: State, view: PdfView, a: Arrange) -> (State, Vec<Effect>) {
    let req = SaveRequest {
        sources: a.sources.clone(),
        plan: a.plan.clone(),
        out: a.name.trim().to_owned(),
        marks: if a.include_marks {
            view.marks.clone()
        } else {
            Vec::new()
        },
        props: properties(&a, &view),
    };
    let arrange = Some(Arrange {
        busy: true,
        message: None,
        ..a
    });
    (
        with(state, PdfView { arrange, ..view }),
        vec![Effect::Pdf(PdfEffect::SaveCopy(req))],
    )
}

fn split(state: State, view: PdfView, a: Arrange) -> (State, Vec<Effect>) {
    let count = view.info.as_ref().map_or(0, |i| i.page_count);
    match parse_page_ranges(&a.ranges, count) {
        Ok(ranges) => {
            let arrange = Some(Arrange {
                busy: true,
                message: None,
                ..a
            });
            let ask = Effect::Pdf(PdfEffect::Split {
                path: view.path.clone(),
                ranges,
            });
            (with(state, PdfView { arrange, ..view }), vec![ask])
        }
        Err(e) => {
            let arrange = Some(Arrange {
                message: Some(e),
                ..a
            });
            (with(state, PdfView { arrange, ..view }), Vec::new())
        }
    }
}

fn finished(state: State, view: PdfView, text: Result<String, String>) -> (State, Vec<Effect>) {
    let (kind, text, arrange) = match text {
        Ok(t) => (NoticeKind::Info, t, None),
        Err(e) => (
            NoticeKind::Error,
            e.clone(),
            view.arrange.clone().map(|a| Arrange {
                busy: false,
                message: Some(e),
                ..a
            }),
        ),
    };
    let notice = Some(Notice { kind, text });
    (
        State {
            notice,
            ..with(state, PdfView { arrange, ..view })
        },
        Vec::new(),
    )
}

fn edit(a: Arrange, msg: &ArrangeMsg, view: &PdfView) -> Option<Arrange> {
    match msg {
        ArrangeMsg::Select(i) => Some(Arrange {
            selected: Some(*i),
            ..a
        }),
        ArrangeMsg::Rotate(by) => rotate(&a, *by),
        ArrangeMsg::Delete => delete(&a),
        ArrangeMsg::Move(by) => shift(&a, *by),
        ArrangeMsg::Undo => {
            let mut undo = a.undo.clone();
            let plan = undo.pop()?;
            Some(Arrange {
                plan,
                undo,
                message: None,
                ..a
            })
        }
        ArrangeMsg::Reset => Some(Arrange {
            sources: vec![view.path.clone()],
            ..start_keeping(&a, view)
        }),
        ArrangeMsg::Name(n) => Some(Arrange {
            name: n.clone(),
            ..a
        }),
        ArrangeMsg::Ranges(r) => Some(Arrange {
            ranges: r.clone(),
            ..a
        }),
        ArrangeMsg::IncludeMarks(on) => Some(Arrange {
            include_marks: *on,
            ..a
        }),
        ArrangeMsg::Property(p, text) => {
            let mut props = a.props.clone();
            props[*p as usize] = text.clone();
            Some(Arrange { props, ..a })
        }
        _ => None,
    }
}

/// Handle the messages about rearranging.
pub fn update(state: State, msg: ArrangeMsg) -> (State, Vec<Effect>) {
    let Some(view) = state.pdf.clone() else {
        return (state, Vec::new());
    };
    let Some(current) = view.arrange.clone() else {
        let arrange = (msg == ArrangeMsg::Start).then(|| start(&view));
        return (with(state, PdfView { arrange, ..view }), Vec::new());
    };
    match msg {
        ArrangeMsg::Cancel => (
            with(
                state,
                PdfView {
                    arrange: None,
                    ..view
                },
            ),
            Vec::new(),
        ),
        ArrangeMsg::Save => save(state, view, current),
        ArrangeMsg::Split => split(state, view, current),
        ArrangeMsg::AddFile => (state, vec![Effect::Pdf(PdfEffect::PickMerge)]),
        ArrangeMsg::FileAdded(Ok(Some((path, pages)))) => {
            let arrange = Some(added(&current, path, pages));
            (with(state, PdfView { arrange, ..view }), Vec::new())
        }
        ArrangeMsg::FileAdded(Ok(None)) => (state, Vec::new()),
        ArrangeMsg::FileAdded(Err(e)) => {
            finished(state, view, Err(format!("Could not use that file: {e}")))
        }
        ArrangeMsg::Saved(Ok(s)) => {
            let skipped = if s.skipped_marks > 0 {
                format!(" {} marks on turned pages were left out.", s.skipped_marks)
            } else {
                String::new()
            };
            finished(
                state,
                PdfView {
                    arrange: None,
                    ..view
                },
                Ok(format!("Saved {} pages to {}.{skipped}", s.pages, s.path)),
            )
        }
        ArrangeMsg::Saved(Err(e)) => finished(state, view, Err(e)),
        ArrangeMsg::Splitted(Ok(names)) => finished(
            state,
            PdfView {
                arrange: None,
                ..view
            },
            Ok(format!(
                "Wrote {} files: {}.",
                names.len(),
                names.join(", ")
            )),
        ),
        ArrangeMsg::Splitted(Err(e)) => finished(state, view, Err(e)),
        other => {
            let arrange = edit(current.clone(), &other, &view).or(Some(current));
            (with(state, PdfView { arrange, ..view }), Vec::new())
        }
    }
}
