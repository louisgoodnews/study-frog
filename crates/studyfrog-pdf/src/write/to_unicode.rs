//! CMap generation for /ToUnicode mapping streams in PDF fonts.

/// Generate a /ToUnicode CMap stream body for WinAnsi / Latin-1 encoding with special glyph mappings.
pub fn generate_to_unicode_cmap() -> Vec<u8> {
    let cmap = r#"/CIDInit /ProcSet findresource begin
12 dict begin
begincmap
/CIDSystemInfo << /Registry (Adobe) /Ordering (UCS) /Supplement 0 >> def
/CMapName /Custom-ToUnicode def
/CMapType 2 def
1 begincodespacerange
<00> <FF>
endcodespacerange
1 beginbfrange
<20> <7E> <0020>
endbfrange
7 beginbfchar
<91> <2018>
<92> <2019>
<93> <201C>
<94> <201D>
<95> <2022>
<96> <2013>
<97> <2014>
endbfchar
1 beginbfrange
<A0> <FF> <00A0>
endbfrange
endcmap
CMapName currentdict /CMap defineresource pop
end
end
"#;
    cmap.as_bytes().to_vec()
}

/// Encode a Unicode string slice into WinAnsi bytes for font rendering.
pub fn encode_text_winansi(text: &str) -> Vec<u8> {
    let mut bytes = Vec::with_capacity(text.len());
    for c in text.chars() {
        let code = c as u32;
        if (32..=126).contains(&code) {
            bytes.push(code as u8);
        } else {
            match c {
                '•' => bytes.push(0x95),
                '–' => bytes.push(0x96),
                '—' => bytes.push(0x97),
                '‘' => bytes.push(0x91),
                '’' => bytes.push(0x92),
                '“' => bytes.push(0x93),
                '”' => bytes.push(0x94),
                c if (160..=255).contains(&code) => bytes.push(c as u8),
                '\n' | '\r' | '\t' => bytes.push(b' '),
                _ => bytes.push(b'?'),
            }
        }
    }
    bytes
}

/// Format bytes as a PDF hexadecimal string enclosed in `<...>` angle brackets.
pub fn bytes_to_pdf_hex(bytes: &[u8]) -> String {
    let mut s = String::with_capacity(bytes.len() * 2 + 2);
    s.push('<');
    for b in bytes {
        use std::fmt::Write;
        let _ = write!(s, "{:02X}", b);
    }
    s.push('>');
    s
}
