//! Tests for the pure parts of the frame-time capture: the CSV text, the summary statistics and the bound.
//! No window and no clock: the durations are synthetic.

use std::path::PathBuf;

use studyfrog_ui::app::frame_times::{MAX_SAMPLES, Recorder, Sample, csv, push, summary};

fn recorder() -> Recorder {
    Recorder {
        path: PathBuf::from("unused.csv"),
        samples: Vec::new(),
        dropped: 0,
    }
}

#[test]
fn csv_of_nothing_is_the_header_only() {
    assert_eq!(csv(&[]), "frame,screen,frame_us\n");
}

#[test]
fn csv_has_one_row_per_sample_in_order() {
    let samples = [
        Sample {
            frame: 0,
            screen: "home",
            frame_us: 1200,
        },
        Sample {
            frame: 1,
            screen: "rehearse",
            frame_us: 16_700,
        },
    ];
    assert_eq!(
        csv(&samples),
        "frame,screen,frame_us\n0,home,1200\n1,rehearse,16700\n"
    );
}

#[test]
fn csv_quotes_a_screen_name_that_would_break_the_row() {
    let samples = [Sample {
        frame: 7,
        screen: "a,b\"c",
        frame_us: 5,
    }];
    assert_eq!(csv(&samples), "frame,screen,frame_us\n7,\"a,b\"\"c\",5\n");
}

#[test]
fn summary_of_nothing_is_none() {
    assert_eq!(summary(&[]), None);
}

#[test]
fn summary_of_one_frame_is_that_frame() {
    let s = summary(&[4200]).expect("one frame");
    assert_eq!(s.frames, 1);
    assert_eq!(s.median_us, 4200);
    assert_eq!(s.p95_us, 4200);
    assert_eq!(s.worst_us, 4200);
}

#[test]
fn median_of_an_odd_count_is_the_middle_frame() {
    let s = summary(&[9, 1, 5]).expect("three frames");
    assert_eq!(s.median_us, 5);
}

#[test]
fn median_of_an_even_count_is_the_mean_of_the_two_middle_frames() {
    let s = summary(&[10, 2, 4, 8]).expect("four frames");
    // sorted: 2 4 8 10; middle two 4 and 8
    assert_eq!(s.median_us, 6);
}

#[test]
fn p95_is_nearest_rank_over_one_to_one_hundred() {
    let durations: Vec<u64> = (1..=100).collect();
    let s = summary(&durations).expect("hundred frames");
    assert_eq!(s.frames, 100);
    assert_eq!(s.p95_us, 95);
    assert_eq!(s.worst_us, 100);
    assert_eq!(s.median_us, 50);
}

#[test]
fn p95_of_twenty_frames_is_the_nineteenth_value() {
    // 95 % of 20 is 19 frames: the 19th smallest value.
    let durations: Vec<u64> = (1..=20).map(|v| v * 10).collect();
    let s = summary(&durations).expect("twenty frames");
    assert_eq!(s.p95_us, 190);
    assert_eq!(s.worst_us, 200);
}

#[test]
fn summary_does_not_depend_on_the_order_of_the_frames() {
    let ordered = [1_u64, 3, 3, 7, 40, 900];
    let shuffled = [900_u64, 3, 40, 1, 7, 3];
    assert_eq!(summary(&ordered), summary(&shuffled));
}

#[test]
fn push_numbers_frames_from_zero_and_keeps_the_screen() {
    let mut rec = recorder();
    push(&mut rec, "home", 100);
    push(&mut rec, "library", 200);
    assert_eq!(rec.samples.len(), 2);
    assert_eq!(rec.samples[0].frame, 0);
    assert_eq!(rec.samples[1].frame, 1);
    assert_eq!(rec.samples[1].screen, "library");
    assert_eq!(rec.samples[1].frame_us, 200);
    assert_eq!(rec.dropped, 0);
}

#[test]
fn push_stops_storing_at_the_bound_and_counts_the_rest() {
    let mut rec = recorder();
    for _ in 0..(MAX_SAMPLES + 3) {
        push(&mut rec, "home", 1);
    }
    assert_eq!(rec.samples.len(), MAX_SAMPLES);
    assert_eq!(rec.dropped, 3);
}
