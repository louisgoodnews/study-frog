#![allow(clippy::unwrap_used, clippy::expect_used)] // tests may panic on failure
//! Pictures out of a PDF land in assets/ under the usual rules.

#[path = "../../studyfrog-pdf/tests/common/mod.rs"]
mod pdfmake;

use std::fs;
use std::path::PathBuf;

use studyfrog_pdf::render::PageRect;
use studyfrog_service::pdf_pictures::{save_page_pictures, save_page_region};
use studyfrog_service::session::{VaultSession, open_vault};

struct Dir(PathBuf);

impl Drop for Dir {
    fn drop(&mut self) {
        let _ = fs::remove_dir_all(&self.0);
    }
}

fn session(label: &str) -> (Dir, VaultSession) {
    let dir = std::env::temp_dir().join(format!("studyfrog-pics-{label}-{}", std::process::id()));
    fs::create_dir_all(dir.join("bio")).unwrap();
    fs::write(dir.join("bio/scan.pdf"), pdfmake::make_scan()).unwrap();
    fs::write(dir.join("bio/three.pdf"), pdfmake::three()).unwrap();
    let (s, _) = open_vault(dir.to_str().unwrap()).unwrap();
    (Dir(dir), s)
}

#[test]
fn the_pictures_of_a_page_are_saved_once_and_a_page_without_pictures_gives_none() {
    let (dir, s) = session("page");
    let saved = save_page_pictures(&s, "bio/scan.pdf", 0).unwrap();
    assert_eq!(saved.len(), 1);
    assert!(
        saved[0].starts_with("assets/scan-p1-1-") && saved[0].ends_with(".png"),
        "{saved:?}"
    );
    let png = fs::read(dir.0.join(&saved[0])).unwrap();
    assert_eq!(&png[..4], b"\x89PNG");
    assert_eq!(
        save_page_pictures(&s, "bio/scan.pdf", 0).unwrap(),
        saved,
        "same bytes, same file"
    );
    assert!(
        save_page_pictures(&s, "bio/three.pdf", 0)
            .unwrap()
            .is_empty()
    );
    assert!(save_page_pictures(&s, "bio/scan.pdf", 7).is_err());
}

#[test]
fn a_rectangle_of_a_page_is_saved_as_a_picture_of_that_size() {
    let (dir, s) = session("region");
    let rect = PageRect {
        x: 50.0,
        y: 60.0,
        width: 100.0,
        height: 40.0,
    };
    let path = save_page_region(&s, "bio/three.pdf", 1, rect).unwrap();
    assert!(path.starts_with("assets/three-p2-snapshot-"), "{path}");
    let img = image::open(dir.0.join(&path)).unwrap();
    assert_eq!((img.width(), img.height()), (200, 80));
    let tiny = PageRect { width: 1.0, ..rect };
    assert!(save_page_region(&s, "bio/three.pdf", 1, tiny).is_err());
    assert!(save_page_region(&s, "../x.pdf", 0, rect).is_err());
}
