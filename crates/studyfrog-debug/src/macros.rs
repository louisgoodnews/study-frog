//! `debug_dump!`: dumps a value at Debug level, labelled with the text of its expression.

/// `debug_dump!(target, value)` calls `dump` with `stringify!(value)` as the label, so the log
/// shows which expression the value came from.
#[macro_export]
macro_rules! debug_dump {
    ($target:expr, $value:expr $(,)?) => {
        $crate::dump($target, ::core::stringify!($value), &($value))
    };
}
