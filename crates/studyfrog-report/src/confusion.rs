//! Interference: pairs of items that are forgotten close together in time and look alike, share a tag
//! or link to each other (STORY-0104). Pure.

/// What is known about an item that lapsed.
#[derive(Debug, Clone, PartialEq, Default)]
pub struct PairInput {
    /// The item, as a number.
    pub item: u64,
    /// Its group (stack).
    pub group: u32,
    /// Its words, hashed, sorted and without repeats (see [`token_set`]).
    pub tokens: Vec<u32>,
    /// Its tags, hashed, sorted and without repeats.
    pub tags: Vec<u32>,
    /// The items it links to, sorted.
    pub links: Vec<u64>,
    /// Days of its lapses (Again after the first review), sorted.
    pub lapse_days: Vec<i64>,
}

/// When two items count as confused.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct PairRules {
    /// Lapse days an item needs to be looked at.
    pub min_lapses: usize,
    /// Days between two lapses that still count as close.
    pub window_days: i64,
    /// Close lapses a pair needs.
    pub min_co_lapses: u32,
    /// Word overlap (Jaccard) that counts as alike.
    pub min_similarity: f32,
    /// At most this many items are compared (those with the most lapses).
    pub max_candidates: usize,
}

/// The defaults: 2 lapses each, within a day of each other, twice; 30 % shared words; 400 items.
pub fn pair_rules_default() -> PairRules {
    PairRules {
        min_lapses: 2,
        window_days: 1,
        min_co_lapses: 2,
        min_similarity: 0.3,
        max_candidates: 400,
    }
}

/// Two items that seem to interfere.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct ConfusionPair {
    /// One item.
    pub a: u64,
    /// The other.
    pub b: u64,
    /// Word overlap, 0 to 1.
    pub similarity: f32,
    /// Lapses of the two within the window of each other.
    pub co_lapses: u32,
    /// Tags both have.
    pub shared_tags: u32,
    /// One links to the other.
    pub linked: bool,
    /// Both are in the same group.
    pub same_group: bool,
    /// How strongly they seem to interfere.
    pub score: f32,
}

/// FNV-1a hash of a word.
fn hash(word: &str) -> u32 {
    word.bytes().fold(0x811c_9dc5_u32, |h, b| {
        (h ^ u32::from(b)).wrapping_mul(0x0100_0193)
    })
}

/// The words of a text (letters and digits, lower case, three characters or more), hashed, sorted, unique.
pub fn token_set(text: &str) -> Vec<u32> {
    let mut out: Vec<u32> = text
        .split(|c: char| !c.is_alphanumeric())
        .filter(|w| w.chars().count() >= 3)
        .map(|w| hash(&w.to_lowercase()))
        .collect();
    out.sort_unstable();
    out.dedup();
    out
}

/// Hashed tag names, sorted and unique.
pub fn tag_set<'a>(tags: impl IntoIterator<Item = &'a str>) -> Vec<u32> {
    let mut out: Vec<u32> = tags.into_iter().map(|t| hash(&t.to_lowercase())).collect();
    out.sort_unstable();
    out.dedup();
    out
}

/// Size of the intersection of two sorted sets.
fn common<T: Ord>(a: &[T], b: &[T]) -> usize {
    let (mut i, mut j, mut n) = (0, 0, 0);
    while i < a.len() && j < b.len() {
        match a[i].cmp(&b[j]) {
            std::cmp::Ordering::Less => i += 1,
            std::cmp::Ordering::Greater => j += 1,
            std::cmp::Ordering::Equal => (i, j, n) = (i + 1, j + 1, n + 1),
        }
    }
    n
}

/// Jaccard similarity of two sorted sets (0 when both are empty).
pub fn jaccard(a: &[u32], b: &[u32]) -> f32 {
    let both = common(a, b);
    let union = a.len() + b.len() - both;
    if union == 0 {
        0.0
    } else {
        both as f32 / union as f32
    }
}

fn near(days: &[i64], d: i64, window: i64) -> bool {
    let i = days.partition_point(|x| *x < d - window);
    days.get(i).is_some_and(|x| *x <= d + window)
}

/// Lapses that happened close together: the smaller of "days of `a` with a lapse of `b` near" and the reverse.
pub fn co_lapses(a: &[i64], b: &[i64], window: i64) -> u32 {
    let one = a.iter().filter(|d| near(b, **d, window)).count();
    let two = b.iter().filter(|d| near(a, **d, window)).count();
    one.min(two) as u32
}

fn judge(a: &PairInput, b: &PairInput, r: &PairRules) -> Option<ConfusionPair> {
    let co = co_lapses(&a.lapse_days, &b.lapse_days, r.window_days);
    if co < r.min_co_lapses {
        return None;
    }
    let similarity = jaccard(&a.tokens, &b.tokens);
    let shared_tags = common(&a.tags, &b.tags) as u32;
    let linked = a.links.binary_search(&b.item).is_ok() || b.links.binary_search(&a.item).is_ok();
    if similarity < r.min_similarity && shared_tags == 0 && !linked {
        return None;
    }
    let context = 0.25 * shared_tags.min(2) as f32 + if linked { 0.5 } else { 0.0 };
    Some(ConfusionPair {
        a: a.item.min(b.item),
        b: a.item.max(b.item),
        similarity,
        co_lapses: co,
        shared_tags,
        linked,
        same_group: a.group == b.group,
        score: co as f32 * (similarity + context),
    })
}

/// Pairs of items that lapse close together and are alike, tagged alike or linked; the strongest first.
pub fn confusion_pairs(items: &[PairInput], rules: &PairRules) -> Vec<ConfusionPair> {
    let mut cand: Vec<&PairInput> = items
        .iter()
        .filter(|i| i.lapse_days.len() >= rules.min_lapses)
        .collect();
    cand.sort_by(|a, b| {
        b.lapse_days
            .len()
            .cmp(&a.lapse_days.len())
            .then(a.item.cmp(&b.item))
    });
    cand.truncate(rules.max_candidates);
    let mut out: Vec<ConfusionPair> = Vec::new();
    for (i, a) in cand.iter().enumerate() {
        out.extend(cand[i + 1..].iter().filter_map(|b| judge(a, b, rules)));
    }
    out.sort_by(|x, y| {
        y.score
            .total_cmp(&x.score)
            .then((x.a, x.b).cmp(&(y.a, y.b)))
    });
    out
}
