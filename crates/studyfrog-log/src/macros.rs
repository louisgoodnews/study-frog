//! Logging macros: `log_error!`, `log_warn!`, `log_info!`, `log_debug!` and `log_trace!`.
//!
//! Every macro has the same form:
//!
//! ```text
//! log_info!(target, "format {}", arg; key = value, key2 = value2)
//! ```
//!
//! - `target` is any `&str` expression, for example `"ui"`.
//! - The format string is a literal, with arguments as for `format!`.
//! - The optional part after `;` lists named fields. Each value is formatted with `Display`.
//!
//! The level is checked before anything is formatted, so a disabled record costs one comparison.

/// Emits a record at the level given as the first argument. Internal: use the `log_*` macros.
#[doc(hidden)]
#[macro_export]
macro_rules! __log_at {
    // A message with no fields.
    ($level:expr, $target:expr, $fmt:literal $(, $arg:expr)*) => {
        if $level <= $crate::level() {
            $crate::emit($level, $target, &::std::format!($fmt $(, $arg)*), &[]);
        }
    };
    // A message with named fields after `;`.
    ($level:expr, $target:expr, $fmt:literal $(, $arg:expr)* ; $($key:ident = $value:expr),+) => {
        if $level <= $crate::level() {
            $crate::emit(
                $level,
                $target,
                &::std::format!($fmt $(, $arg)*),
                &[$((::core::stringify!($key), ::std::format!("{}", $value).as_str())),+],
            );
        }
    };
}

/// Logs at `Error` level. See the module documentation for the form.
#[macro_export]
macro_rules! log_error {
    ($($args:tt)+) => {
        $crate::__log_at!($crate::Level::Error, $($args)+)
    };
}

/// Logs at `Warn` level. See the module documentation for the form.
#[macro_export]
macro_rules! log_warn {
    ($($args:tt)+) => {
        $crate::__log_at!($crate::Level::Warn, $($args)+)
    };
}

/// Logs at `Info` level. See the module documentation for the form.
#[macro_export]
macro_rules! log_info {
    ($($args:tt)+) => {
        $crate::__log_at!($crate::Level::Info, $($args)+)
    };
}

/// Logs at `Debug` level. See the module documentation for the form.
#[macro_export]
macro_rules! log_debug {
    ($($args:tt)+) => {
        $crate::__log_at!($crate::Level::Debug, $($args)+)
    };
}

/// Logs at `Trace` level. See the module documentation for the form.
#[macro_export]
macro_rules! log_trace {
    ($($args:tt)+) => {
        $crate::__log_at!($crate::Level::Trace, $($args)+)
    };
}
