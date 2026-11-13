---
id: BUG-0014
type: bug
title: "CSV importer fails on unquoted multiline fields in AI-generated CSV files"
status: done
epic: EPIC-031
parents: []
estimate: 2
labels: [m11, ext, service]
acceptance_criteria:
  - "Headed CSV files containing unquoted multiline fields (such as numbered lists in the back column) merge continuation lines into the preceding record instead of failing with 'a card needs at least a front and a back'"
  - "Both the extension delimited importer (studyfrog-ext-example) and the service CSV mapper (studyfrog-service::xfer_csv_map) import the psychologische-diagnostik benchmark file cleanly with all 15 cards intact"
---

## Context
When importing an AI-generated CSV file (`stack,kind,title,front,back,tags`) where a card's `back` field contains an unquoted multiline numbered list across lines 4–7:
```csv
psychologische-diagnostik,card,,What are the four main reasons why we need accurate psychological diagnostics?,1. Accurate description of human experience and behavior
2. Testing psychological theories explaining human experience and behavior
3. Psychologically grounded prediction and decision making
4. Evaluation of predictions and psychological measures,objectives
```
StudyFrog failed to import the file with:
`Could not create the cards: Tab-separated text (front, back, tags): line 5: a card needs at least a front and a back`

## Description
Root cause:
Because the LLM did not wrap the multiline `back` field in double quotes `"..."`, `parse_rows` in `examples/studyfrog-ext-example/src/delimited.rs` and `rows` in `crates/studyfrog-service/src/xfer_csv.rs` treated lines 5, 6, and 7 as standalone rows with 1, 1, and 2 fields respectively instead of continuation lines of the 6-column record started on line 4. When `parse_delimited` inspected line 5 (`fields.len() == 1`), it rejected the file because `fields.len() <= m.front.max(m.back)`.

## Acceptance Criteria
- [x] Headed CSV files containing unquoted multiline fields (such as numbered lists in the back column) merge continuation lines into the preceding record instead of failing with 'a card needs at least a front and a back'
- [x] Both the extension delimited importer (studyfrog-ext-example) and the service CSV mapper (studyfrog-service::xfer_csv_map) import the psychologische-diagnostik benchmark file cleanly with all 15 cards intact

## Design Notes
- In both `examples/studyfrog-ext-example/src/delimited.rs` and `crates/studyfrog-service/src/xfer_csv_map.rs`, before applying `normalize_row`, merge continuation rows in headed comma-separated files (`delim == ',' && cols >= 3`) whenever the previous row has fewer than `cols` fields or the current row has fewer than `cols` fields, joining the first field of the continuation row onto the last field of the previous row with `\n` and appending any remaining fields.

## Dependencies
None.

## Open Questions
None.
