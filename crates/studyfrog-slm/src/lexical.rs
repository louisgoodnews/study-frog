//! A runtime that needs no model: signed feature hashing of words, word pairs and character
//! trigrams. It measures how much surface text two strings share, not what they mean, so a
//! paraphrase scores lower than it deserves. It is deterministic, instant and offline, which makes
//! it the default until a semantic model is installed.

use crate::runtime::{SlmError, SlmRuntime};
use crate::vector::normalize;

/// Word matching rates a good paraphrase around 0.5 (measured on a labelled set of 44 answers, see
/// the scoring spec), so a cosine of 0.44 is what counts as acceptable.
pub const LEXICAL_PIVOT: f32 = 0.44;
/// Output dimensions: enough to keep collisions rare for short answers.
pub const DIMS: usize = 512;
const WORD_WEIGHT: f32 = 1.0;
const PAIR_WEIGHT: f32 = 0.7;
const TRIGRAM_WEIGHT: f32 = 0.4;

/// The lexical runtime. It has no state; the type exists to implement [`SlmRuntime`].
#[derive(Debug, Clone, Copy, Default)]
pub struct Lexical;

fn fnv1a(bytes: &[u8], seed: u64) -> u64 {
    bytes.iter().fold(0xcbf2_9ce4_8422_2325 ^ seed, |h, b| {
        (h ^ u64::from(*b)).wrapping_mul(0x0100_0000_01b3)
    })
}

/// Add one feature with a hash-derived bucket and sign.
fn add(v: &mut [f32], feature: &str, weight: f32) {
    let h = fnv1a(feature.as_bytes(), 0);
    let sign = if fnv1a(feature.as_bytes(), 1) & 1 == 0 {
        1.0
    } else {
        -1.0
    };
    v[(h % DIMS as u64) as usize] += sign * weight;
}

fn words(text: &str) -> Vec<String> {
    text.split(|c: char| !c.is_alphanumeric())
        .filter(|w| !w.is_empty())
        .map(str::to_lowercase)
        .collect()
}

fn add_trigrams(v: &mut [f32], word: &str) {
    let padded: Vec<char> = std::iter::once('^')
        .chain(word.chars())
        .chain(std::iter::once('$'))
        .collect();
    for w in padded.windows(3) {
        add(v, &w.iter().collect::<String>(), TRIGRAM_WEIGHT);
    }
}

/// The embedding of one text.
pub fn lexical_vector(text: &str) -> Vec<f32> {
    let mut v = vec![0.0; DIMS];
    let ws = words(text);
    for (i, w) in ws.iter().enumerate() {
        add(&mut v, w, WORD_WEIGHT);
        add_trigrams(&mut v, w);
        if let Some(next) = ws.get(i + 1) {
            add(&mut v, &format!("{w} {next}"), PAIR_WEIGHT);
        }
    }
    normalize(&mut v);
    v
}

/// Embed every text.
pub fn lexical_embed(texts: &[&str]) -> Vec<Vec<f32>> {
    texts.iter().map(|t| lexical_vector(t)).collect()
}

impl SlmRuntime for Lexical {
    fn name(&self) -> &str {
        "word matching (built in)"
    }

    fn pivot(&self) -> f32 {
        LEXICAL_PIVOT
    }

    fn embed(&self, texts: &[&str]) -> Result<Vec<Vec<f32>>, SlmError> {
        Ok(lexical_embed(texts))
    }
}
