//! The selection of a choice item in a rehearsal run (STORY-0222). Pure: each function returns
//! the new flags, one per option in the order written, and never touches the run.

use studyfrog_core::uid::Uid;

/// The options picked on one item. `picked` belongs to `item` only; another item reads as empty.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct ChoiceSelection {
    /// The item the flags were picked on.
    pub item: Option<Uid>,
    /// One flag per option; true when the option is picked.
    pub picked: Vec<bool>,
}

/// The flags of `item` with `len` options: the picked ones when they belong to it, else all off.
pub fn picked_for(selection: &ChoiceSelection, item: Uid, len: usize) -> Vec<bool> {
    if selection.item == Some(item) && selection.picked.len() == len {
        selection.picked.clone()
    } else {
        vec![false; len]
    }
}

/// `picked` with option `index` flipped. An index outside the options leaves it unchanged.
pub fn toggled(picked: &[bool], index: usize) -> Vec<bool> {
    let mut out = picked.to_vec();
    if let Some(flag) = out.get_mut(index) {
        *flag = !*flag;
    }
    out
}

/// Single choice: exactly option `index` is picked, replacing any earlier pick.
pub fn only(len: usize, index: usize) -> Vec<bool> {
    (0..len).map(|i| i == index).collect()
}

/// The flags after a click or key on option `index`: toggled for multiple select, replaced for
/// single choice.
pub fn pick(picked: &[bool], index: usize, multiple: bool) -> Vec<bool> {
    if multiple {
        toggled(picked, index)
    } else {
        only(picked.len(), index)
    }
}

/// Flags with exactly the options at `indices` (0-based) picked. `None` when an index is outside
/// the `len` options, so the selection never holds a wrong option.
pub fn from_indices(len: usize, indices: &[usize]) -> Option<Vec<bool>> {
    if indices.iter().any(|&i| i >= len) {
        return None;
    }
    Some((0..len).map(|i| indices.contains(&i)).collect())
}

/// Whether Submit is enabled: at least one option is picked.
pub fn can_submit(picked: &[bool]) -> bool {
    picked.iter().any(|&on| on)
}

/// How many options are picked.
pub fn chosen_count(picked: &[bool]) -> usize {
    picked.iter().filter(|&&on| on).count()
}

/// The option a 1-based number (key or control word) names, if it is one of `len` options.
pub fn option_for_number(n: usize, len: usize) -> Option<usize> {
    (1..=len).contains(&n).then(|| n - 1)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn toggle_flips_one_option_and_ignores_out_of_range() {
        assert_eq!(toggled(&[false, false, true], 0), vec![true, false, true]);
        assert_eq!(toggled(&[true, false], 0), vec![false, false]);
        assert_eq!(toggled(&[true, false], 5), vec![true, false]);
    }

    #[test]
    fn single_select_replaces_the_earlier_pick() {
        let first = pick(&[false, false, false], 0, false);
        assert_eq!(first, vec![true, false, false]);
        assert_eq!(pick(&first, 2, false), vec![false, false, true]);
    }

    #[test]
    fn multiple_select_keeps_earlier_picks() {
        let both = pick(&pick(&[false; 3], 0, true), 2, true);
        assert_eq!(both, vec![true, false, true]);
        assert_eq!(pick(&both, 0, true), vec![false, false, true]);
    }

    #[test]
    fn submit_needs_a_pick() {
        assert!(!can_submit(&[false, false]));
        assert!(can_submit(&[false, true]));
        assert!(!can_submit(&[]));
    }

    #[test]
    fn from_indices_rejects_an_index_outside_the_options() {
        assert_eq!(from_indices(3, &[0, 2]), Some(vec![true, false, true]));
        assert_eq!(from_indices(3, &[3]), None);
        assert_eq!(from_indices(3, &[]), Some(vec![false; 3]));
    }

    #[test]
    fn numbers_map_to_options_in_range_only() {
        assert_eq!(option_for_number(1, 3), Some(0));
        assert_eq!(option_for_number(3, 3), Some(2));
        assert_eq!(option_for_number(4, 3), None);
        assert_eq!(option_for_number(0, 3), None);
    }

    #[test]
    fn selection_of_another_item_reads_as_empty() {
        let sel = ChoiceSelection {
            item: Some(Uid(1)),
            picked: vec![true, false],
        };
        assert_eq!(picked_for(&sel, Uid(1), 2), vec![true, false]);
        assert_eq!(picked_for(&sel, Uid(2), 2), vec![false, false]);
        assert_eq!(picked_for(&sel, Uid(1), 3), vec![false; 3]);
    }
}
