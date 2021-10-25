//! Experimental version of `std::error::Report` proposal.
//!
//! The `Error::source` method on the `Error` trait is one way in which error chains may be exposed
//! and interacted with. However, currently, the method only returns the top-most error in the
//! chain, such that the rest of the context encapsulated in the error chain (including the root
//! error itself) are not presented.
//!
//! This module defines a `Report` type that exposes the entire error chain, not just the top-most
//! error. The `Report` type also exposes options for formatting the error chain (currently either
//! as a single line, or in a multi-line format with each cause in the error chain on a new line).
//!
//! Note that this `Report` type, which we'll refer to as `std::Report`, is not meant as a
//! replacement for types like `eyre::Report` or `anyhow::Error`. `std::Report` is much more
//! minimal in comparison; its only job is to expose an interface for formatting errors that you
//! want to print. `eyre::Report` is able to store an error and additional context. It also
//! supports custom user-defined output formats, while `std::Report` only makes available a limited
//! set of formatting options that are intended to be sensible defaults for the most common error
//! handling use cases.
//!
//! A report of an error with 0 sources looks like this:
//!
//! ```rust
//! # use std::fmt;
//! # use std::error::Error;
//! # use trial_and_error::Report;
//!
//! # #[derive(Debug)]
//! # struct SuperErrorSideKick;
//! #
//! # impl fmt::Display for SuperErrorSideKick {
//! #     fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
//! #         write!(f, "SuperErrorSideKick is here!")
//! #     }
//! # }
//! #
//! # impl Error for SuperErrorSideKick {}
//!
//! fn main() {
//!     let report = Report::new(SuperErrorSideKick).pretty(true);
//!     
//!     println!("{}", report);
//! }
//! ```
//!
//! ```console
//! SuperErrorSideKick is here!
//! ```
//!
//! Note that `std::Report` only requires that the wrapped error implements the `Error` trait.
//! It doesn't require that the wrapped error be `Send` or `Sync`:
//!
//! ```rust
//! #![feature(negative_impls)]
//! # use std::fmt;
//! # use std::error::Error;
//!
//! # use trial_and_error::Report;
//!
//! impl !Send for SuperError {}
//! impl !Sync for SuperError {}
//!
//! # #[derive(Debug)]
//! # struct SuperError {
//! #     side: SuperErrorSideKick,
//! # }
//! #
//! # impl fmt::Display for SuperError {
//! #     fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
//! #         write!(f, "SuperError is here!")
//! #     }
//! # }
//! #
//! # impl Error for SuperError {
//! #     fn source(&self) -> Option<&(dyn Error + 'static)> {
//! #         Some(&self.side)
//! #     }
//! # }
//! #
//! # #[derive(Debug)]
//! # struct SuperErrorSideKick;
//! #
//! # impl fmt::Display for SuperErrorSideKick {
//! #     fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
//! #         write!(f, "SuperErrorSideKick is here!")
//! #     }
//! # }
//! #
//! # impl Error for SuperErrorSideKick {}
//!
//! fn main() {
//!     let report = Report::new(SuperError { side: SuperErrorSideKick });
//!
//!     println!("{}", report);
//! }
//! ```
//!
//! It also is not required that the wrapped error be `'static`.
//!
//! ```rust
//! # use std::fmt;
//! # use std::error::Error;
//!
//! # use trial_and_error::Report;
//!
//! #[derive(Debug)]
//! struct SuperError<'a> {
//!     side: &'a str
//! }
//!
//! impl<'a> fmt::Display for SuperError<'a> {
//!     fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
//!         write!(f, "SuperError is here: {}", self.side)
//!     }
//! }
//!
//! impl<'a> Error for SuperError<'a> {}
//!
//! fn main() {
//!     let msg = String::from("Huzzah!");
//!     let mut report = Report::new(SuperError { side: &msg });
//!
//!     println!("{}", report);
//! }
//! ```

use std::{
    error::Error,
    fmt::{self, Write},
};

/// The main `Report` type.
pub struct Report<E> {
    /// The error being reported.
    error: E,
    /// Whether the full backtrace should be included as part of the report.
    show_backtrace: bool,
    /// Whether the report should be pretty printed.
    pretty: bool,
}

impl<E> Report<E>
where
    E: Error,
{
    /// Create a new `Report` from an input error.
    pub fn new(error: E) -> Report<E> {
        Report {
            error,
            show_backtrace: false,
            pretty: false,
        }
    }

    /// Enable pretty-printing the report.
    pub fn pretty(mut self, pretty: bool) -> Self {
        self.pretty = pretty;
        self
    }

    /// Enable showing a backtrace for the report.
    pub fn show_backtrace(mut self, show_backtrace: bool) -> Self {
        self.show_backtrace = show_backtrace;
        self
    }

    /// Format the report as a single line.
    fn fmt_singleline(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.error)?;

        let sources = self.error.source().into_iter().flat_map(<dyn Error>::chain);

        for cause in sources {
            write!(f, ": {}", cause)?;
        }

        Ok(())
    }

    /// Format the report as multiple lines, with each error cause on its own line.
    fn fmt_multiline(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let error = &self.error;

        write!(f, "{}", error)?;

        if let Some(cause) = error.source() {
            write!(f, "\n\nCaused by:")?;

            let multiple = cause.source().is_some();

            for (ind, error) in cause.chain().enumerate() {
                writeln!(f)?;

                let format = if multiple { Some(ind) } else { None };
                let mut indented = Indented {
                    buffer: f,
                    needs_indent: true,
                    format,
                };

                write!(indented, "{}", error)?;
            }
        }

        if self.show_backtrace {
            let backtrace = error.backtrace();

            if let Some(backtrace) = backtrace {
                let backtrace = backtrace.to_string();

                f.write_str("\n\nStack backtrace:\n")?;
                f.write_str(backtrace.trim_end())?;
            }
        }

        Ok(())
    }
}

impl<E> From<E> for Report<E>
where
    E: Error,
{
    fn from(error: E) -> Self {
        Report::new(error)
    }
}

impl<E> fmt::Display for Report<E>
where
    E: Error,
{
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        if self.pretty {
            self.fmt_multiline(f)
        } else {
            self.fmt_singleline(f)
        }
    }
}

// This type intentionally outputs the same format for `Display` and `Debug`for
// situations where you unwrap a `Report` or return it from main.
impl<E> fmt::Debug for Report<E>
where
    E: Error,
{
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        fmt::Display::fmt(self, f)
    }
}

/// Encapsulates how error sources are indented and formatted.
struct Indented<'a, D: ?Sized> {
    /// The write buffer that is written to.
    buffer: &'a mut D,
    /// Whether the output needs to be indented or not.
    needs_indent: bool,
    /// Details regarding how the output should be formatted.
    /// `Some(usize)` indicates that the output should be numbered due to the error chain having
    /// multiple causes.
    /// `None` indicates that the error chain has at most one cause, and can thus be formatted in a
    /// more uniform manner.
    format: Option<usize>,
}

impl<D> Write for Indented<'_, D>
where
    D: Write + ?Sized,
{
    fn write_str(&mut self, s: &str) -> fmt::Result {
        for (ind, line) in s.split('\n').enumerate() {
            if ind > 0 {
                self.buffer.write_char('\n')?;
                self.needs_indent = true;
            }

            if self.needs_indent {
                if line.is_empty() {
                    continue;
                }

                match self.format {
                    Some(size) => {
                        if ind == 0 {
                            write!(self.buffer, "{: >4}: ", size)?;
                        } else {
                            write!(self.buffer, "      ")?;
                        }
                    }
                    None => write!(self.buffer, "    ")?,
                }

                self.needs_indent = false;
            }

            self.buffer.write_str(line)?;
        }

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    use std::error::Error;
    use std::fmt;

    #[derive(Debug)]
    struct GenericError<D> {
        message: D,
        source: Option<Box<dyn Error + 'static>>,
    }

    impl<D> GenericError<D> {
        fn new(message: D) -> GenericError<D> {
            GenericError {
                message,
                source: None,
            }
        }

        fn new_with_source<E>(message: D, source: E) -> GenericError<D>
        where
            E: Error + 'static,
        {
            let source: Box<dyn Error + 'static> = Box::new(source);
            let source = Some(source);
            GenericError { message, source }
        }
    }

    impl<D> fmt::Display for GenericError<D>
    where
        D: fmt::Display,
    {
        fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
            fmt::Display::fmt(&self.message, f)
        }
    }

    impl<D> Error for GenericError<D>
    where
        D: fmt::Debug + fmt::Display,
    {
        fn source(&self) -> Option<&(dyn Error + 'static)> {
            self.source.as_deref()
        }
    }

    #[derive(Debug)]
    struct SuperError {
        side: SuperErrorSideKick,
    }

    impl fmt::Display for SuperError {
        fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
            write!(f, "SuperError is here!")
        }
    }

    impl Error for SuperError {
        fn source(&self) -> Option<&(dyn Error + 'static)> {
            Some(&self.side)
        }
    }

    #[derive(Debug)]
    struct SuperErrorSideKick;

    impl fmt::Display for SuperErrorSideKick {
        fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
            write!(f, "SuperErrorSideKick is here!")
        }
    }

    impl Error for SuperErrorSideKick {}

    #[test]
    fn single_line_formatting() {
        let error = SuperError {
            side: SuperErrorSideKick,
        };
        let report = Report::new(&error);
        let expected = String::from("SuperError is here!: SuperErrorSideKick is here!");

        assert_eq!(expected, report.to_string(),);
    }

    #[test]
    fn multi_line_formatting() {
        let error = SuperError {
            side: SuperErrorSideKick,
        };
        let report = Report::new(&error).pretty(true);
        let expected =
            String::from("SuperError is here!\n\nCaused by:\n    SuperErrorSideKick is here!");

        assert_eq!(expected, report.to_string(),);
    }

    #[test]
    fn error_with_no_sources_formats_single_line_correctly() {
        let report = Report::new(SuperErrorSideKick);
        let expected = String::from("SuperErrorSideKick is here!");

        assert_eq!(expected, report.to_string());
    }

    #[test]
    fn error_with_no_sources_formats_multi_line_correctly() {
        let report = Report::new(SuperErrorSideKick).pretty(true);
        let expected = String::from("SuperErrorSideKick is here!");

        assert_eq!(expected, report.to_string());
    }

    #[test]
    fn error_formats_with_rude_display_impl() {
        #[derive(Debug)]
        struct MyMessage;
        impl std::fmt::Display for MyMessage {
            fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                f.write_str("line 1\nline 2")?;
                f.write_str("\nline 3\nline 4\n")?;
                f.write_str("line 5\nline 6")?;
                Ok(())
            }
        }

        let error = GenericError::new(MyMessage);
        let error = GenericError::new_with_source(MyMessage, error);
        let error = GenericError::new_with_source(MyMessage, error);
        let error = GenericError::new_with_source(MyMessage, error);
        let report = Report::new(error).pretty(true);
        let expected = r#"line 1
line 2
line 3
line 4
line 5
line 6

Caused by:
   0: line 1
      line 2
      line 3
      line 4
      line 5
      line 6
   1: line 1
      line 2
      line 3
      line 4
      line 5
      line 6
   2: line 1
      line 2
      line 3
      line 4
      line 5
      line 6"#;

        let actual = report.to_string();
        pretty_assertions::assert_eq!(expected, actual);
    }

    #[test]
    #[ignore]
    fn error_with_backtrace_outputs_correctly() {
        use std::backtrace::Backtrace;

        #[derive(Debug)]
        struct ErrorWithBacktrace<'a> {
            msg: &'a str,
            trace: Backtrace,
        }

        impl<'a> fmt::Display for ErrorWithBacktrace<'a> {
            fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
                write!(f, "Error with backtrace: {}", self.msg)
            }
        }

        impl<'a> Error for ErrorWithBacktrace<'a> {
            fn backtrace(&self) -> Option<&Backtrace> {
                Some(&self.trace)
            }
        }

        let msg = String::from("The source of the error");
        let report = Report::new(ErrorWithBacktrace {
            msg: &msg,
            trace: Backtrace::capture(),
        })
        .pretty(true)
        .show_backtrace(true);

        let expected = String::from(
            "Error with backtrace: The source of the error\n\nStack backtrace:\ndisabled backtrace",
        );

        assert_eq!(expected, report.to_string());
    }

    #[test]
    fn multiple_error_sources() {}
}
