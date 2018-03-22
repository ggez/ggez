#[cfg(not(feature = "use-log-crate"))]
/// Provides drop-in replacements for macros from [`log`] crate.
///
/// Default to wrapping [`println!`], or become equivalent to those found in [`log`] crate
/// if `use-log-crate` feature is enabled.
///
/// These are intended to be used throughout `ggez` and dependant libraries, allowing executables
/// to opt-in for a [`log`]-dependant solution, or stick to the `std` macro wrapper.
///
/// WITH `use-log-crate` feature: contains a fake re-export (a spoof) of [`log::Level`],
/// to enable use of general `log!` macro (`ggez_log!` here) by dependant libraries.
///
/// WITHOUT `use-log-crate` feature: re-exports [`log::Level`].
///
/// [`log`]: https://docs.rs/log/0.4.1/log/
/// [`log::Level`]: https://docs.rs/log/0.4.1/log/enum.Level.html
/// [`println!`]: https://doc.rust-lang.org/std/macro.println.html
pub mod feature_dependant {
    use std::cmp;
    use std::fmt;

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
    static LOG_LEVEL_NAMES: [&'static str; 6] = ["OFF", "ERROR", "WARN", "INFO", "DEBUG", "TRACE"];

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

    #[macro_export]
    macro_rules! ggez_log {
        (target: $target:expr, $lvl:expr, $($arg:tt)+) => (println!("{} {}", $target, $lvl));
        ($lvl:expr, $($arg:tt)+) => (ggez_log!(target: module_path!(), $lvl, $($arg)+))
    }

    #[macro_export]
    macro_rules! ggez_error {
        (target: $target:expr, $($arg:tt)*) => (
            ggez_log!(target: $target, logging::Level::Error, $($arg)*);
        );
        ($($arg:tt)*) => (
            ggez_log!(logging::Level::Error, $($arg)*);
        )
    }
}

#[cfg(feature = "use-log-crate")]
/// Provides drop-in replacements for macros from [`log`] crate.
///
/// Default to wrapping [`println!`], or become equivalent to those found in [`log`] crate
/// if `use-log-crate` feature is enabled.
///
/// These are intended to be used throughout `ggez` and dependant libraries, allowing executables
/// to opt-in for a [`log`]-dependant solution, or stick to the `std` macro wrapper.
///
/// WITH `use-log-crate` feature: contains a fake re-export (a spoof) of [`log::Level`],
/// to enable use of general `log!` macro (`ggez_log!` here) by dependant libraries.
///
/// WITHOUT `use-log-crate` feature: re-exports [`log::Level`].
///
/// [`log`]: https://docs.rs/log/0.4.1/log/
/// [`log::Level`]: https://docs.rs/log/0.4.1/log/enum.Level.html
/// [`println!`]: https://doc.rust-lang.org/std/macro.println.html
pub mod feature_dependant {
    extern crate log;
    pub use self::log::Level;

    #[macro_export]
    macro_rules! ggez_log {
        (target: $target:expr, $lvl:expr, $($arg:tt)+) => (log!(target: $target, $lvl, $($arg)+));
        ($lvl:expr, $($arg:tt)+) => (log!(target: module_path!(), $lvl, $($arg)+))
    }

    #[macro_export]
    macro_rules! ggez_error {
        (target: $target:expr, $($arg:tt)*) => (
            log!(target: $target, logging::Level::Error, $($arg)*);
        );
        ($($arg:tt)*) => (
            log!(logging::Level::Error, $($arg)*);
        )
    }
}
