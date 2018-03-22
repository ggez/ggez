//! Provides opt-in replacements for macros from [`log`] crate.
//!
//! Macros default to expanding into [`println!`], formatted as `[target][LEVEL] message`.
//! With `use-log-crate` feature they expand into their equivalent macros from [`log`] crate.
//!
//! These are intended to be used throughout `ggez` and dependant libraries, allowing executables
//! to opt-in for a [`log`]-dependant logging solution, or stick to the `std` [`println!`].
//!
//! Executables intending to use [`log`] crate (and `use-log-crate` feature) can (should)
//! use the original macros.
//!
//! WITH `use-log-crate` feature: contains a fake re-export (a spoof) of [`log::Level`],
//! to enable use of general [`log!`] macro (`ggez_log!` here) by dependant libraries.
//!
//! WITHOUT `use-log-crate` feature: re-exports [`log::Level`].
//!
//! [`log`]: https://docs.rs/log/0.4.1/log/
//! [`log!`]: https://docs.rs/log/0.4.1/log/macro.log.html
//! [`log::Level`]: https://docs.rs/log/0.4.1/log/enum.Level.html
//! [`println!`]: https://doc.rust-lang.org/std/macro.println.html

#[cfg(not(feature = "use-log-crate"))]
/// Provides a partial re-implementation of [`log::Level`].
///
/// [`log::Level`]: https://docs.rs/log/0.4.1/log/enum.Level.html
pub mod no_log_crate {
    use std::cmp;
    use std::fmt;

    static LOG_LEVEL_NAMES: [&'static str; 6] = ["OFF", "ERROR", "WARN", "INFO", "DEBUG", "TRACE"];

    /// Spoofs logging verbosity levels of [`log`] crate.
    /// Only needed to properly mimic usage of general `log!` macro.
    /// [`log`]: https://docs.rs/log/0.4.1/log/
    #[repr(usize)]
    #[derive(Copy, Eq, Debug, Hash)]
    pub enum Level {
        /// Designates very serious errors.
        Error = 1,
        /// Designates hazardous situations.
        Warn,
        /// Designates useful information.
        Info,
        /// Designates lower priority information.
        Debug,
        /// Designates very low priority, often extremely verbose, information.
        Trace,
    }

    impl fmt::Display for Level {
        fn fmt(&self, fmt: &mut fmt::Formatter) -> fmt::Result {
            fmt.pad(LOG_LEVEL_NAMES[*self as usize])
        }
    }

    impl PartialOrd for Level {
        #[inline]
        fn partial_cmp(&self, other: &Level) -> Option<cmp::Ordering> {
            Some(self.cmp(other))
        }
    }

    impl Ord for Level {
        #[inline]
        fn cmp(&self, other: &Level) -> cmp::Ordering {
            (*self as usize).cmp(&(*other as usize))
        }
    }

    impl PartialEq for Level {
        #[inline]
        fn eq(&self, other: &Level) -> bool {
            *self as usize == *other as usize
        }
    }

    impl Clone for Level {
        #[inline]
        fn clone(&self) -> Level {
            *self
        }
    }
}

#[cfg(not(feature = "use-log-crate"))]
pub use self::no_log_crate::Level;

#[cfg(feature = "use-log-crate")]
extern crate log;
#[cfg(feature = "use-log-crate")]
pub use self::log::Level;

#[cfg(not(feature = "use-log-crate"))]
/// General logging macro.
///
/// Log with the specified `Level` and [`format!`] based argument list.
///
/// [`format!`]: https://doc.rust-lang.org/std/macro.format.html
#[macro_export]
macro_rules! ggez_log {
    (target: $target:expr, $lvl:expr, $($arg:tt)+) => (
        println!("[{}][{}] {}", $target, $lvl, format!($($arg)*))
    );
    ($lvl:expr, $($arg:tt)+) => (ggez_log!(target: module_path!(), $lvl, $($arg)+))
}

#[cfg(feature = "use-log-crate")]
/// General logging macro.
///
/// Log with the specified `Level` and [`format!`] based argument list.
///
/// [`format!`]: https://doc.rust-lang.org/std/macro.format.html
#[macro_export]
macro_rules! ggez_log {
    (target: $target:expr, $lvl:expr, $($arg:tt)+) => (log!(target: $target, $lvl, $($arg)+));
    ($lvl:expr, $($arg:tt)+) => (log!(target: module_path!(), $lvl, $($arg)+))
}

#[cfg(not(feature = "use-log-crate"))]
/// In [`log`] crate, determines if a message logged at the specified level in that module will
/// be logged. Without `use-log-crate` feature simply resolves to `true`.
///
/// This can be used to avoid expensive computation of log message arguments if
/// the message would be ignored anyway.
///
/// [`log`]: https://docs.rs/log/0.4.1/log/
#[macro_export]
macro_rules! ggez_log_enabled {
    (target: $target: expr, $lvl: expr) => {
        true
    };
    ($lvl: expr) => {
        true
    };
}

#[cfg(feature = "use-log-crate")]
/// In [`log`] crate, determines if a message logged at the specified level in that module will
/// be logged. Without `use-log-crate` feature simply resolves to `true`.
///
/// This can be used to avoid expensive computation of log message arguments if
/// the message would be ignored anyway.
///
/// [`log`]: https://docs.rs/log/0.4.1/log/
#[macro_export]
macro_rules! ggez_log_enabled {
    (target: $target: expr, $lvl: expr) => {
        log_enabled!(target: $target, $lvl)
    };
    ($lvl: expr) => {
        log_enabled!(target: module_path!(), $lvl)
    };
}

/// Logs a message at the error level. Argument list is identical to [`format!`].
///
/// [`format!`]: https://doc.rust-lang.org/std/macro.format.html
#[macro_export]
macro_rules! ggez_error {
    (target: $target:expr, $($arg:tt)*) => (
        ggez_log!(target: $target, logging::Level::Error, $($arg)*);
    );
    ($($arg:tt)*) => (
        ggez_log!(logging::Level::Error, $($arg)*);
    )
}

/// Logs a message at the warn level. Argument list is identical to [`format!`].
///
/// [`format!`]: https://doc.rust-lang.org/std/macro.format.html
#[macro_export]
macro_rules! ggez_warn {
    (target: $target:expr, $($arg:tt)*) => (
        ggez_log!(target: $target, logging::Level::Warn, $($arg)*);
    );
    ($($arg:tt)*) => (
        ggez_log!(logging::Level::Warn, $($arg)*);
    )
}

/// Logs a message at the info level. Argument list is identical to [`format!`].
///
/// [`format!`]: https://doc.rust-lang.org/std/macro.format.html
#[macro_export]
macro_rules! ggez_info {
    (target: $target:expr, $($arg:tt)*) => (
        ggez_log!(target: $target, logging::Level::Info, $($arg)*);
    );
    ($($arg:tt)*) => (
        ggez_log!(logging::Level::Info, $($arg)*);
    )
}

/// Logs a message at the debug level. Argument list is identical to [`format!`].
///
/// [`format!`]: https://doc.rust-lang.org/std/macro.format.html
#[macro_export]
macro_rules! ggez_debug {
    (target: $target:expr, $($arg:tt)*) => (
        ggez_log!(target: $target, logging::Level::Debug, $($arg)*);
    );
    ($($arg:tt)*) => (
        ggez_log!(logging::Level::Debug, $($arg)*);
    )
}

/// Logs a message at the trace level. Argument list is identical to [`format!`].
///
/// [`format!`]: https://doc.rust-lang.org/std/macro.format.html
#[macro_export]
macro_rules! ggez_trace {
    (target: $target:expr, $($arg:tt)*) => (
        ggez_log!(target: $target, logging::Level::Trace, $($arg)*);
    );
    ($($arg:tt)*) => (
        ggez_log!(logging::Level::Trace, $($arg)*);
    )
}
