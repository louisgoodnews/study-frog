---
id: STORY-0085
type: story
title: "Performance optimization in StudyFrog pdf-viewer"
status: done
epic: EPIC-028
parents: [EPIC-028]
estimate: 5
labels: [m12, pdf, perf]
acceptance_criteria:
  - "PDF page rasterization and rendering operate with smooth scrolling and panning without dropping UI frame rates below 60 fps"
  - "Rendered page textures are cached efficiently in memory and purged according to a bounded LRU cache"
  - "Zoom changes reuse pre-rasterized tiles or trigger background re-renders without freezing the main application UI thread"
  - "Thumbnail rendering in the sidebar uses virtualized requests rather than queueing renders for every page at once"
---

## Context
Users viewing dense textbooks, multi-page slide decks, or scanned documents encounter sluggish navigation and memory spikes in the StudyFrog PDF viewer, especially when navigating quickly across pages or adjusting zoom levels.

## Description
Optimize the performance and memory footprint of the PDF viewer across `crates/studyfrog-pdf` and `crates/studyfrog-ui`:
- Virtualize sidebar page thumbnail rendering (`Side::Pages` in `pdf_side.rs`) so that only visible page thumbnails are requested.
- Reuse hayro's `RenderCache` across page renders in an active document session rather than instantiating a fresh cache per page.
- Segregate thumbnail picture caches from full-size page texture caches in `pdf_load.rs` to prevent cache eviction thrashing.
- Bound memory consumption with an explicit LRU texture cache.
- Maintain a 60 fps frame rate during scrolling and zooming by keeping rasterization off the main thread.

## Acceptance Criteria
- [x] PDF page rasterization and rendering operate with smooth scrolling and panning without dropping UI frame rates below 60 fps
- [x] Rendered page textures are cached efficiently in memory and purged according to a bounded LRU cache
- [x] Zoom changes reuse pre-rasterized tiles or trigger background re-renders without freezing the main application UI thread
- [x] Thumbnail rendering in the sidebar uses virtualized requests rather than queueing renders for every page at once

## Design Notes
Crate touchpoints:
- `crates/studyfrog-ui/src/view/pdf.rs`, `pdf_page.rs`, `pdf_texture.rs`, `pdf_side.rs`, and `pdf_load.rs`.
- `crates/studyfrog-pdf/src/render.rs` and `limits.rs`: retain `hayro::RenderCache` across calls in worker session state.
- Segregate thumbnail caching from active reading page textures to stop thrashing.
- Add Criterion benchmarks in `crates/studyfrog-pdf/benches/` for multi-page rasterization throughput.
- Release target: version 0.5.0 (Phase M12).

## Progress (2026-11-08)
Done and verified:
- **LRU by last use** (`crates/studyfrog-ui/src/pdf_cache.rs`): `view.pictures` is ordered from least to most recently used; a picture drawn on screen that has fallen into the older half of its kind is moved to the end (`PdfMsg::Used`, sent by the page view, the thumbnail list and the arrange grid), so a steady screen sends nothing, and trimming drops the least recently used. Pages (6) and thumbnails (60) are bounded apart. Unit tests: the page read again stays while an older-used one goes, a thumbnail kept in view survives 300 others, only the older half counts as stale.
- **60 fps** (`crates/studyfrog-ui/tests/pdf_frame_time.rs`): a 500-page PDF in the real update and view, driven through a bare `egui::Context` with real wheel events; each measured frame is the update, layout and tessellation (all CPU work of a frame; only the GPU upload is missing). At 200 % with the thumbnail list open, 90 frames of scrolling the thumbnails, wheel-panning the page down and sideways and standing still, in an *unoptimised test build*: median 8.3 ms, p95 9.3 ms, worst 9.9 ms against the 16.7 ms budget of 60 fps (the test fails above 16.7 ms in an optimised build or 50 ms in a debug one). Renders run on the worker and never in a frame.
- **Zoom changes** re-render in the background: the frame after a zoom change asks the worker for the sharp picture, keeps showing the old one and stays within the budget (test `a_zoom_change_asks_the_worker_...`).
- **Virtualised thumbnails**: with 500 pages, 4 thumbnails are asked for at first and 44 after scrolling down 43 rows, never more than one new one per frame; a steady screen asks for nothing (tests in `pdf_frame_time.rs`).
- The page scale is capped so a picture never exceeds the graphics card's texture side or the renderer's 36-megapixel budget (before, 400 % on a 2x display failed with "too big" and a 2048-texel card got an oversized texture).

Re-measured (`cargo bench -p studyfrog-pdf --bench pdf`, 500-page text file, a busy machine): parsing 0.52 ms, a page at 1x 8.7 ms, at 2x 11.1 ms. Parsing is about 6 % of a 1x render, so a per-session parsed document and `RenderCache` would save about half a millisecond per render on the worker, never on the UI thread; it stays unbuilt (`render.rs` unchanged). Not measured: files with embedded fonts and images. Not built: tiled rendering (zoom re-renders whole pages in the background instead, which the criterion allows).

## Dependencies
EPIC-028.

## Open Questions
None.
