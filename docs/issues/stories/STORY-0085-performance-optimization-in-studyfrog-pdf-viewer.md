---
id: STORY-0085
type: story
title: Performance optimization in StudyFrog pdf-viewer
status: in_progress
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
- [ ] PDF page rasterization and rendering operate with smooth scrolling and panning without dropping UI frame rates below 60 fps
- [ ] Rendered page textures are cached efficiently in memory and purged according to a bounded LRU cache
- [ ] Zoom changes reuse pre-rasterized tiles or trigger background re-renders without freezing the main application UI thread
- [ ] Thumbnail rendering in the sidebar uses virtualized requests rather than queueing renders for every page at once

## Design Notes
Crate touchpoints:
- `crates/studyfrog-ui/src/view/pdf.rs`, `pdf_page.rs`, `pdf_texture.rs`, `pdf_side.rs`, and `pdf_load.rs`.
- `crates/studyfrog-pdf/src/render.rs` and `limits.rs`: retain `hayro::RenderCache` across calls in worker session state.
- Segregate thumbnail caching from active reading page textures to stop thrashing.
- Add Criterion benchmarks in `crates/studyfrog-pdf/benches/` for multi-page rasterization throughput.
- Release target: version 0.5.0 (Phase M12).

## Progress (2026-11-08)
Done and tested (unit tests in `pdf_load.rs`, benchmarks in `crates/studyfrog-pdf/benches/pdf.rs`):
- Full-size page pictures and thumbnails are trimmed apart (6 pages, 60 thumbnails, oldest first within each kind), so scrolling the thumbnails cannot push out the page being read, and at most six multi-megabyte page textures are held instead of forty. The thumbnail scale is one constant (250) for the sidebar and the arrange grid.
- Sidebar thumbnails are requested only for rows in view (`is_rect_visible` around the `Need`); read from the code, not covered by a test.
- Renders already run on the worker thread, not in the frame.

Measured on the generated 500-page text file (`cargo bench -p studyfrog-pdf --bench pdf`): parsing the document 0.59 ms, a thumbnail 2.8 ms, a page at 1x 5.7 ms, at 2x 10.3 ms. Parsing again on every render is about 10 % of a 1x render, so keeping a parsed document and `RenderCache` per session would save well under a millisecond and is not built. Not measured: documents with embedded fonts and images, where the share may be larger.

Open: eviction is by age, not by last use (no real LRU); the 60 fps claim and smooth scrolling are not checked in a real window; `render.rs` is unchanged.

## Dependencies
EPIC-028.

## Open Questions
None.
