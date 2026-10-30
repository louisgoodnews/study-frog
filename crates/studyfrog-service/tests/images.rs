#![allow(clippy::unwrap_used, clippy::expect_used)] // tests may panic on failure
//! Adding pictures to a vault.

use std::fs;

use studyfrog_service::images::add_image;
use studyfrog_service::session::open_vault;

#[test]
fn a_picture_is_copied_once_and_only_pictures_are_accepted() {
    let base = std::env::temp_dir().join(format!("studyfrog-images-{}", std::process::id()));
    let (vault, outside) = (base.join("vault"), base.join("outside"));
    fs::create_dir_all(&vault).unwrap();
    fs::create_dir_all(&outside).unwrap();
    let (session, _) = open_vault(vault.to_str().unwrap()).unwrap();
    let png = outside.join("My Map.PNG");
    fs::write(&png, b"\x89PNG not really").unwrap();
    let first = add_image(&session, &png).unwrap();
    assert!(
        first.starts_with("assets/my-map-") && first.ends_with(".png"),
        "{first}"
    );
    assert_eq!(fs::read(vault.join(&first)).unwrap(), b"\x89PNG not really");
    assert_eq!(
        add_image(&session, &png).unwrap(),
        first,
        "same content, same copy"
    );
    fs::write(&png, b"other").unwrap();
    assert_ne!(
        add_image(&session, &png).unwrap(),
        first,
        "other content, other name"
    );

    let text = outside.join("notes.txt");
    fs::write(&text, "x").unwrap();
    assert!(add_image(&session, &text).is_err());
    assert!(add_image(&session, &outside.join("missing.png")).is_err());
    let _ = fs::remove_dir_all(&base);
}
