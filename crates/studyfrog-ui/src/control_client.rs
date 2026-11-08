//! `studyfrog ctl [--socket PATH] <command> [args...]`: send one control request to a running
//! instance and print its answer (and, for `logs --follow`, every log event after it) as JSON lines.

use std::io::{self, BufRead, BufReader, Write};
use std::path::{Path, PathBuf};

use serde_json::Value;

/// The socket to use and the words of the command.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CtlArgs {
    /// `--socket PATH`, else `<app home>/run/control.sock`.
    pub socket: Option<PathBuf>,
    /// The command and its arguments.
    pub words: Vec<String>,
}

/// Read the arguments after `ctl`.
pub fn parse_ctl_args(args: &[String]) -> Result<CtlArgs, String> {
    let (socket, words) = match args {
        [flag, path, rest @ ..] if flag == "--socket" => (Some(PathBuf::from(path)), rest),
        [flag] if flag == "--socket" => return Err("--socket needs a path".into()),
        rest => (None, rest),
    };
    if words.is_empty() {
        return Err(
            "give a command, such as `studyfrog ctl state` (`studyfrog ctl help` lists them)"
                .into(),
        );
    }
    Ok(CtlArgs {
        socket,
        words: words.to_vec(),
    })
}

/// The request line for the words: the first word is the command, the rest its arguments, so no
/// quoting is lost on the way.
pub fn request_line(words: &[String]) -> String {
    let (cmd, args) = words
        .split_first()
        .map_or(("", &[][..]), |(c, a)| (c.as_str(), a));
    serde_json::json!({"id": 1, "cmd": cmd, "args": args}).to_string()
}

/// Whether a printed line ends the exchange: an answer that does not start a follow.
fn ends(line: &str) -> Option<bool> {
    let v: Value = serde_json::from_str(line).ok()?;
    let ok = v.get("ok")?.as_bool()?;
    let following = v.pointer("/result/following").and_then(Value::as_bool) == Some(true);
    Some(!(ok && following))
}

#[cfg(unix)]
/// Send `line` to the socket and copy the answer (and events that follow it) to `out`; whether the
/// answer was a success.
pub fn exchange(socket: &Path, line: &str, out: &mut impl Write) -> io::Result<bool> {
    let stream = std::os::unix::net::UnixStream::connect(socket).map_err(|e| {
        io::Error::new(
            e.kind(),
            format!("{}: {e} (is StudyFrog running?)", socket.display()),
        )
    })?;
    writeln!(&stream, "{line}")?;
    let mut ok = false;
    for got in BufReader::new(&stream).lines() {
        let got = got?;
        writeln!(out, "{got}")?;
        out.flush()?;
        if let Some(done) = ends(&got) {
            ok = got.contains("\"ok\":true");
            if done {
                break;
            }
        }
    }
    Ok(ok)
}

#[cfg(not(unix))]
fn exchange(_: &Path, _: &str, _: &mut impl Write) -> io::Result<bool> {
    Err(io::Error::other("the control socket needs a Unix system"))
}

/// Run `ctl`; the exit code (0 when the answer was a success).
pub fn run(args: &[String]) -> u8 {
    let result = parse_ctl_args(args).and_then(|a| {
        let socket = a
            .socket
            .or_else(|| crate::runtime::paths().map(|p| p.run.join("control.sock")))
            .ok_or("no application home is known")?;
        exchange(&socket, &request_line(&a.words), &mut io::stdout().lock())
            .map_err(|e| e.to_string())
    });
    match result {
        Ok(true) => 0,
        Ok(false) => 1,
        Err(e) => {
            eprintln!("studyfrog ctl: {e}");
            2
        }
    }
}
