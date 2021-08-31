//! Experimental version of `std::error::Report` proposal.
//! TODO: add link to RFC once it's been written.
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
//! replacement for types like `eyre::Report` or `anyhow::Error`. `std::Report` is much more minimal in comparison; its only job
//! is to expose an interface for formatting errors that you want to print. `eyre::Report` is able
//! to additionally store an error and additional context. It also supports custom user-defined
//! output formats, while `std::Report` only makes available a single formatting option that is
//! intended to be a sensible default for more error handling use cases. 
//!
//! # Examples
//! 
//! Let's say we're given the following error setup:
//! ```rust
//! use std::fmt;
//! use std::error::Error;
//! 
//! use trial_and_error::Report;
//! 
//! #[derive(Debug)]
//! struct SuperError {
//!     side: SuperErrorSidekick,
//! }
//! 
//! impl fmt::Display for SuperError {
//!     fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
//!         write!(f, "SuperError is here!")
//!     }
//! }
//! 
//! impl Error for SuperError {
//!     fn source(&self) -> Option<&(dyn Error + 'static)> {
//!         Some(&self.side)
//!     }
//! }
//! 
//! #[derive(Debug)]
//! struct SuperErrorSidekick;
//! 
//! impl fmt::Display for SuperErrorSidekick {
//!     fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
//!         write!(f, "SuperErrorSidekick is here!")
//!     }
//! }
//! 
//! impl Error for SuperErrorSidekick {}
//! ```
//! 
//! We can wrap an instance of our `SuperError` type in a `Report` and print it out in single-line
//! format:
//!
//! ```rust
//! # use std::fmt;
//! # use std::error::Error;
//! # 
//! # use trial_and_error::Report;
//! # 
//! # #[derive(Debug)]
//! # struct SuperError {
//! #     side: SuperErrorSidekick,
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
//! # struct SuperErrorSidekick;
//! # 
//! # impl fmt::Display for SuperErrorSidekick {
//! #     fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
//! #         write!(f, "SuperErrorSidekick is here!")
//! #     }
//! # }
//! # 
//! # impl Error for SuperErrorSidekick {}
//!
//! fn main() {
//!     // `Report` can be given a borrowed error
//!     let error = SuperError { side: SuperErrorSidekick };
//!     let mut report = Report::new(&error);
//!
//!     // It can also be given an owned error
//!     // let mut report = Report::new(SuperError { side: SuperErrorSidekick });
//!     
//!     println!("{}", report);
//! }
//! ```
//!
//! This prints:
//!
//! ```console
//! SuperError is here!: SuperErrorSidekick is here!
//! ```
//!
//! Or we can print it out in multiline format by specifying the `pretty` option:
//!
//! ```rust
//! # use std::fmt;
//! # use std::error::Error;
//! # 
//! # use trial_and_error::Report;
//! # 
//! # #[derive(Debug)]
//! # struct SuperError {
//! #     side: SuperErrorSidekick,
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
//! # struct SuperErrorSidekick;
//! # 
//! # impl fmt::Display for SuperErrorSidekick {
//! #     fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
//! #         write!(f, "SuperErrorSidekick is here!")
//! #     }
//! # }
//! # 
//! # impl Error for SuperErrorSidekick {}
//!
//! fn main() {
//!     let mut report = Report::new(SuperError { side: SuperErrorSidekick });
//!     let pretty_report = report.pretty(true);
//!     
//!     println!("{}", pretty_report);
//! }
//! ```
//!
//! This prints:
//!
//! ```console
//! SuperError is here!
//!
//! Caused by:
//!     SuperErrorSidekick is here!
//! ```
//! 
//! A report of an error with 0 sources looks like this:
//! 
//! ```rust
//! # use std::fmt;
//! # use std::error::Error;
//! # use trial_and_error::Report; 
//! 
//! # #[derive(Debug)]
//! # struct SuperErrorSidekick;
//! # 
//! # impl fmt::Display for SuperErrorSidekick {
//! #     fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
//! #         write!(f, "SuperErrorSidekick is here!")
//! #     }
//! # }
//! # 
//! # impl Error for SuperErrorSidekick {}
//! 
//! fn main() {
//!     let mut report = Report::new(SuperErrorSidekick);
//!     let pretty_report = report.pretty(true);
//!     
//!     println!("{}", pretty_report);
//! }
//! ```
//! 
//! ```console
//! SuperErrorSidekick is here!
//! ```
//!
//! Note that `std::Report` only requires that the wrapped error implements the `Error` trait.
//! It doesn't require that the wrapped error be `Send`, `Sync`, or `'static`:
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
//! #     side: SuperErrorSidekick,
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
//! # struct SuperErrorSidekick;
//! # 
//! # impl fmt::Display for SuperErrorSidekick {
//! #     fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
//! #         write!(f, "SuperErrorSidekick is here!")
//! #     }
//! # }
//! # 
//! # impl Error for SuperErrorSidekick {}
//!
//! fn main() {
//!     let mut report = Report::new(SuperError { side: SuperErrorSidekick });
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
    source: E,
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
    pub fn new(source: E) -> Report<E> {
        Report {
            source,
            show_backtrace: false,
            pretty: false,
        }
    }
    
    /// Sets the report's `pretty` flag.
    pub fn pretty(&mut self, pretty: bool) -> &mut Report<E> {
        self.pretty = pretty;
        self
    }

    /// Sets the report's `show_backtrace` flag.
    pub fn show_backtrace(&mut self, show_backtrace: bool) -> &mut Report<E> {
        self.show_backtrace = show_backtrace;
        self
    }
    
    /// Format the report as a single line.
    fn fmt_singleline(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.source)?;

        let sources = self
            .source
            .source()
            .into_iter()
            .flat_map(<dyn Error>::chain);

        for cause in sources {
            write!(f, ": {}", cause)?;
        }

        Ok(())
    }

    /// Format the report as multiple lines, with each error cause on its own line.
    fn fmt_multiline(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let error = &self.source;

        write!(f, "{}", error)?;

        if let Some(cause) = error.source() {
            write!(f, "\n\nCaused by:")?;

            let multiple = cause.source().is_some();
            let format = if multiple {
                Format::Numbered { ind: 0 }
            } else {
                Format::Uniform {
                    indentation: "    ",
                }
            };

            for error in cause.chain() {
                writeln!(f)?;
                let mut indented = Indented {
                    inner: f,
                    needs_indent: true,
                    format,
                };
                write!(indented, "{}", error)?;
            }
        }

        if self.show_backtrace {
            let backtrace = error.backtrace();
            if let Some(backtrace) = backtrace {
                let mut backtrace = backtrace.to_string();
                write!(f, "\n\n")?;
                writeln!(f, "Stack backtrace:")?;
                backtrace.truncate(backtrace.trim_end().len());
                write!(f, "{}", backtrace)?;
            }
        }

        Ok(())
    }
}

// is it possible to make this work for types that convert into the inner error type?
impl<E> From<E> for Report<E>
where
    E: Error,
{
    fn from(source: E) -> Self {
        Report::new(source)
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

// This type intentionally outputs the same format for `Display` and `Debug` for
// situations where you `unwrap` a `Report` or return it from main.
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
    inner: &'a mut D,
    needs_indent: bool,
    format: Format,
}

/// The possible variants that error sources can be formatted as.
#[derive(Clone, Copy)]
enum Format {
    /// Insert uniform indentation before every line.
    ///
    /// This format takes a static string as input and inserts it after every newline.
    Uniform {
        /// The string to insert as indentation.
        indentation: &'static str,
    },
    /// Inserts a number before the first line.
    ///
    /// This format hard codes the indentation level to match the indentation from
    /// `std::backtrace::Backtrace`.
    Numbered {
        /// The index to insert before the first line of output.
        ind: usize,
    },
}

impl<D> Write for Indented<'_, D>
where
    D: Write + ?Sized,
{
    fn write_str(&mut self, s: &str) -> fmt::Result {
        for (ind, line) in s.split('\n').enumerate() {
            if ind > 0 {
                self.inner.write_char('\n')?;
                self.needs_indent = true;
            }

            if self.needs_indent {
                // Don't render the line unless its actually got text on it
                if line.is_empty() {
                    continue;
                }

                self.format.insert_indentation(ind, &mut self.inner)?;
                self.needs_indent = false;
            }

            self.inner.write_fmt(format_args!("{}", line))?;
        }

        Ok(())
    }
}

impl Format {
    /// Write the specified formatting to the write buffer.
    fn insert_indentation(&mut self, line: usize, f: &mut dyn Write) -> fmt::Result {
        match self {
            Format::Uniform { indentation } => {
                write!(f, "{}", indentation)
            }
            Format::Numbered { ind } => {
                if line == 0 {
                    write!(f, "{: >4}: ", ind)?;
                    *ind += 1;
                    Ok(())
                } else {
                    write!(f, "      ")
                }
            }
        }
    }
}
