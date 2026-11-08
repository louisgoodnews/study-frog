#![allow(dead_code, unused_imports, clippy::unwrap_used, clippy::expect_used)] // shared test helpers
//! A 500-page PDF open in the viewer, driven frame by frame through a bare `egui::Context`
//! (STORY-0085): real input events, real layout and tessellation, the real worker for pictures.

#[path = "../../../studyfrog-pdf/tests/common/mod.rs"]
mod pdfmake;

use std::fs;
use std::time::Instant;

use eframe::egui::{self, Event, Modifiers, MouseWheelSource, MouseWheelUnit, TouchPhase};

use crate::common::{TempDir, run_to_idle, temp_dir};
use studyfrog_ui::effect::Effect;
use studyfrog_ui::exec::{Worker, worker_new};
use studyfrog_ui::model::{Model, apply, frame};
use studyfrog_ui::msg::Msg;
use studyfrog_ui::pdf_types::{PageKey, PdfEffect, PdfMsg, PdfView, is_thumb};

pub const PAGES: usize = 500;
pub const SIZE: egui::Vec2 = egui::vec2(1200.0, 800.0);
/// Where the pointer rests over the thumbnail list (the side panel right of the navigation rail).
pub const OVER_SIDE: egui::Pos2 = egui::pos2(200.0, 450.0);

pub struct Frames {
    pub dir: TempDir,
    pub model: Model,
    pub worker: Worker,
    pub ctx: egui::Context,
    pub time: f64,
    /// The window size in points.
    pub size: egui::Vec2,
    /// The graphics card's largest texture side (8192 is common; egui's own default is 2048).
    pub max_side: usize,
}

/// The frame budget the tests hold to: 60 fps (16.7 ms) in an optimised build; an unoptimised
/// test build is several times slower, so it gets three times as long.
pub fn budget_ms() -> f64 {
    if cfg!(debug_assertions) { 50.0 } else { 16.7 }
}

fn big_pdf() -> Vec<u8> {
    let page: pdfmake::Page = (0..40)
        .map(|l| {
            let y = 760.0 - l as f32 * 18.0;
            (
                "Retrieval practice beats rereading and spacing slows forgetting",
                72.0,
                y,
            )
        })
        .collect();
    pdfmake::make(&vec![page; PAGES], Some("Big"))
}

/// A vault with `bio/big.pdf` (500 pages of text) open in the viewer at its first page.
pub fn open_big(label: &str) -> Frames {
    let dir = temp_dir(label);
    fs::create_dir_all(dir.path.join("bio")).unwrap();
    fs::write(dir.path.join("bio/big.pdf"), big_pdf()).unwrap();
    let mut model = Model::default();
    model.state.app.tutorial_offered = true;
    let mut worker = worker_new(None);
    let open = Effect::OpenVault(dir.path.display().to_string());
    run_to_idle(&mut model, &mut worker, vec![open]);
    let mut f = Frames {
        dir,
        model,
        worker,
        ctx: egui::Context::default(),
        time: 0.0,
        size: SIZE,
        max_side: 8192,
    };
    send(&mut f, PdfMsg::Open("bio/big.pdf".into(), 0));
    assert_eq!(view(&f).info.as_ref().unwrap().page_count, PAGES);
    f
}

/// Where the pointer rests over the page being read.
pub const OVER_PAGE: egui::Pos2 = egui::pos2(800.0, 500.0);

pub fn wheel_over_side(dy: f32) -> Vec<Event> {
    wheel_at(OVER_SIDE, egui::vec2(0.0, dy))
}

/// A wheel turn at `at`: `delta` moves the content (negative y scrolls down).
pub fn wheel_at(at: egui::Pos2, delta: egui::Vec2) -> Vec<Event> {
    vec![
        Event::PointerMoved(at),
        Event::MouseWheel {
            unit: MouseWheelUnit::Point,
            delta,
            phase: TouchPhase::Move,
            source: MouseWheelSource::Unknown,
            modifiers: Modifiers::NONE,
        },
    ]
}

/// The pictures the effects ask the worker for.
pub fn renders(fx: &[Effect]) -> Vec<PageKey> {
    fx.iter()
        .filter_map(|e| match e {
            Effect::Pdf(PdfEffect::Render { key, .. }) => Some(*key),
            _ => None,
        })
        .collect()
}

pub fn thumb_renders(fx: &[Effect]) -> Vec<usize> {
    renders(fx)
        .into_iter()
        .filter(|k| is_thumb(*k))
        .map(|k| k.page)
        .collect()
}

pub fn view(f: &Frames) -> &PdfView {
    f.model.state.pdf.as_ref().unwrap()
}

pub fn send(f: &mut Frames, m: PdfMsg) {
    let fx = apply(&mut f.model, Msg::Pdf(m));
    run_to_idle(&mut f.model, &mut f.worker, fx);
}

/// Let the worker carry out the effects and deliver what it made.
pub fn run(f: &mut Frames, fx: Vec<Effect>) {
    run_to_idle(&mut f.model, &mut f.worker, fx);
}

pub fn frame_of(f: &mut Frames, events: Vec<Event>) -> Vec<Effect> {
    timed_frame(f, events).0
}

/// One frame: input, update and layout, then tessellation; the time (ms) is for all of it.
pub fn timed_frame(f: &mut Frames, events: Vec<Event>) -> (Vec<Effect>, f64) {
    f.time += 1.0 / 60.0;
    let input = egui::RawInput {
        screen_rect: Some(egui::Rect::from_min_size(egui::Pos2::ZERO, f.size)),
        time: Some(f.time),
        max_texture_side: Some(f.max_side),
        events,
        ..Default::default()
    };
    let start = Instant::now();
    let mut fx = Vec::new();
    let model = &mut f.model;
    let mut out = f.ctx.run_ui(input, |ui| fx.extend(frame(model, ui)));
    out.textures_delta.clear(); // no graphics card to upload to
    let shapes = f.ctx.tessellate(out.shapes, out.pixels_per_point);
    let ms = start.elapsed().as_secs_f64() * 1000.0;
    assert!(!shapes.is_empty());
    (fx, ms)
}

/// Deliver what `first` asks for, then draw and deliver until the screen asks for nothing more.
pub fn settle(f: &mut Frames, first: Vec<Effect>) {
    run(f, first);
    for _ in 0..30 {
        let fx = frame_of(f, Vec::new());
        if renders(&fx).is_empty() && view(f).requested.is_empty() {
            return;
        }
        run(f, fx);
    }
    panic!("the screen did not settle");
}

/// A picture painted in a frame: the texture's size in texels and the painted rectangle in points.
#[derive(Debug, Clone, Copy)]
pub struct Painted {
    pub texels: [usize; 2],
    pub rect: egui::Rect,
}

fn collect(shape: &egui::Shape, out: &mut Vec<(egui::TextureId, egui::Rect)>) {
    match shape {
        egui::Shape::Vec(v) => v.iter().for_each(|s| collect(s, out)),
        egui::Shape::Rect(r) => out.extend(r.brush.as_ref().map(|b| (b.fill_texture_id, r.rect))),
        egui::Shape::Mesh(m) => out.push((m.texture_id, m.calc_bounds())),
        _ => {}
    }
}

/// One frame, returning the user textures (pictures, not the font atlas) it painted.
pub fn painted(f: &mut Frames) -> (Vec<Effect>, Vec<Painted>) {
    f.time += 1.0 / 60.0;
    let input = egui::RawInput {
        screen_rect: Some(egui::Rect::from_min_size(egui::Pos2::ZERO, f.size)),
        time: Some(f.time),
        max_texture_side: Some(f.max_side),
        ..Default::default()
    };
    let mut fx = Vec::new();
    let model = &mut f.model;
    let mut out = f.ctx.run_ui(input, |ui| fx.extend(frame(model, ui)));
    out.textures_delta.clear();
    let mut found = Vec::new();
    out.shapes
        .iter()
        .for_each(|c| collect(&c.shape, &mut found));
    let manager = f.ctx.tex_manager();
    let textures = manager.read();
    let pictures = found
        .into_iter()
        .filter(|(id, _)| matches!(id, egui::TextureId::Managed(n) if *n > 0))
        .filter_map(|(id, rect)| {
            let texels = textures.meta(id)?.size;
            Some(Painted { texels, rect })
        })
        .collect();
    (fx, pictures)
}

/// The largest picture on screen (the page being read), and its texels per physical pixel across.
pub fn page_sharpness(f: &mut Frames) -> (Painted, f32) {
    let (fx, pictures) = painted(f);
    assert!(renders(&fx).is_empty(), "settled before measuring: {fx:?}");
    let page = pictures
        .into_iter()
        .max_by(|a, b| a.rect.area().total_cmp(&b.rect.area()))
        .expect("a page picture is painted");
    let physical = page.rect.width() * f.ctx.pixels_per_point();
    (page, page.texels[0] as f32 / physical)
}
