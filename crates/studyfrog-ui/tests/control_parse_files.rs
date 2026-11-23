#![allow(clippy::unwrap_used, clippy::expect_used)] // tests may panic on failure
//! The control protocol's parser for the file commands: `import` destinations and `pdf find`.

use studyfrog_ui::control_cmd::*;
use studyfrog_ui::control_parse::parse_request;

fn cmd(line: &str) -> Command {
    parse_request(line).unwrap().cmd
}

fn err(line: &str) -> String {
    parse_request(line).unwrap_err().1
}

#[test]
fn import_destinations_are_one_of_root_stack_or_new_stack() {
    assert_eq!(
        cmd("import a.csv"),
        Command::Import {
            path: "a.csv".into(),
            stack: None,
            root: false,
            new_stack: None
        }
    );
    assert_eq!(
        cmd("import a.csv --root"),
        Command::Import {
            path: "a.csv".into(),
            stack: None,
            root: true,
            new_stack: None
        }
    );
    assert_eq!(
        cmd("import a.csv --new-stack Biology"),
        Command::Import {
            path: "a.csv".into(),
            stack: None,
            root: false,
            new_stack: Some("Biology".into())
        }
    );
    assert!(err("import a.csv --root --stack french").contains("destination"));
    assert_eq!(
        cmd("pdf find reliability --file lecture.pdf"),
        Command::PdfFind(Some("lecture.pdf".into()), "reliability".into())
    );
    assert_eq!(
        cmd("pdf find test score"),
        Command::PdfFind(None, "test score".into())
    );
    assert!(err("pdf find").contains("text"));
    assert!(err("pdf move a.pdf").contains("find"));
}
