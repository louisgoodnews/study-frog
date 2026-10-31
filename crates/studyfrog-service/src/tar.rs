//! A minimal tar (ustar) writer and reader, so a backup opens with any archive tool. Regular files
//! only; names of up to 255 bytes (split into prefix and name at a slash).

use std::io::{self, Read, Write};

const BLOCK: usize = 512;

fn octal(value: u64, width: usize) -> Vec<u8> {
    let mut text = format!("{value:0w$o}", w = width - 1).into_bytes();
    text.push(0);
    text
}

fn put(header: &mut [u8; BLOCK], at: usize, bytes: &[u8]) {
    header[at..at + bytes.len()].copy_from_slice(bytes);
}

/// Where to cut `name` into a prefix (at most 155 bytes) and a name (at most 100), if it must be.
fn split_name(name: &str) -> io::Result<(&str, &str)> {
    if name.len() <= 100 {
        return Ok(("", name));
    }
    let too_long = || {
        io::Error::new(
            io::ErrorKind::InvalidInput,
            format!("the path is too long for a backup: {name}"),
        )
    };
    name.match_indices('/')
        .map(|(i, _)| (&name[..i], &name[i + 1..]))
        .find(|(prefix, rest)| prefix.len() <= 155 && rest.len() <= 100)
        .ok_or_else(too_long)
}

/// Write the header of a file of `size` bytes; its content (and padding) must follow.
pub fn tar_header(w: &mut impl Write, name: &str, size: u64, mtime: u64) -> io::Result<()> {
    let (prefix, base) = split_name(name)?;
    let mut h = [0_u8; BLOCK];
    put(&mut h, 0, base.as_bytes());
    put(&mut h, 100, &octal(0o644, 8));
    put(&mut h, 108, &octal(0, 8));
    put(&mut h, 116, &octal(0, 8));
    put(&mut h, 124, &octal(size, 12));
    put(&mut h, 136, &octal(mtime, 12));
    put(&mut h, 148, b"        ");
    h[156] = b'0';
    put(&mut h, 257, b"ustar\0");
    put(&mut h, 263, b"00");
    put(&mut h, 345, prefix.as_bytes());
    let sum: u32 = h.iter().map(|b| u32::from(*b)).sum();
    put(&mut h, 148, &octal(u64::from(sum), 7));
    h[155] = b' ';
    w.write_all(&h)
}

/// Zero bytes that bring `size` bytes of content up to a whole block.
pub fn tar_padding(w: &mut impl Write, size: u64) -> io::Result<()> {
    let rest = (size % BLOCK as u64) as usize;
    if rest > 0 {
        w.write_all(&vec![0; BLOCK - rest])?;
    }
    Ok(())
}

/// A file whose content is in memory.
pub fn tar_bytes(w: &mut impl Write, name: &str, data: &[u8], mtime: u64) -> io::Result<()> {
    tar_header(w, name, data.len() as u64, mtime)?;
    w.write_all(data)?;
    tar_padding(w, data.len() as u64)
}

/// The end of the archive: two empty blocks.
pub fn tar_finish(w: &mut impl Write) -> io::Result<()> {
    w.write_all(&[0; BLOCK * 2])
}

/// The name and size of the next file of an archive.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TarEntry {
    /// Full path inside the archive.
    pub name: String,
    /// Content length in bytes.
    pub size: u64,
}

fn text_of(field: &[u8]) -> String {
    let end = field.iter().position(|b| *b == 0).unwrap_or(field.len());
    String::from_utf8_lossy(&field[..end]).into_owned()
}

fn number_of(field: &[u8]) -> io::Result<u64> {
    let text = text_of(field);
    u64::from_str_radix(text.trim(), 8).map_err(|_| {
        io::Error::new(
            io::ErrorKind::InvalidData,
            "a number in the archive is not valid",
        )
    })
}

/// Read the next header; `None` at the end of the archive. Entries that are not regular files are
/// an error (a backup never contains them).
pub fn tar_next(r: &mut impl Read) -> io::Result<Option<TarEntry>> {
    let mut h = [0_u8; BLOCK];
    r.read_exact(&mut h)?;
    if h.iter().all(|b| *b == 0) {
        return Ok(None);
    }
    let mut check = h;
    check[148..156].copy_from_slice(b"        ");
    let sum: u64 = check.iter().map(|b| u64::from(*b)).sum();
    if number_of(&h[148..156])? != sum {
        return Err(io::Error::new(
            io::ErrorKind::InvalidData,
            "the archive is damaged (a header does not add up)",
        ));
    }
    if !matches!(h[156], b'0' | 0) {
        return Err(io::Error::new(
            io::ErrorKind::InvalidData,
            "the archive holds something other than files",
        ));
    }
    let (prefix, name) = (text_of(&h[345..500]), text_of(&h[0..100]));
    let name = if prefix.is_empty() {
        name
    } else {
        format!("{prefix}/{name}")
    };
    Ok(Some(TarEntry {
        name,
        size: number_of(&h[124..136])?,
    }))
}

/// Copy the content of `entry` to `w` and skip its padding.
pub fn tar_copy(r: &mut impl Read, entry: &TarEntry, w: &mut impl Write) -> io::Result<()> {
    let copied = io::copy(&mut r.take(entry.size), w)?;
    if copied != entry.size {
        return Err(io::Error::new(
            io::ErrorKind::UnexpectedEof,
            "the archive ends in the middle of a file",
        ));
    }
    let rest = (entry.size % BLOCK as u64) as usize;
    if rest > 0 {
        io::copy(&mut r.take((BLOCK - rest) as u64), &mut io::sink())?;
    }
    Ok(())
}
