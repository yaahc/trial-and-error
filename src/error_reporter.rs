//! Experimental version of `std::error::Report` proposal

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

    /// Format the report as multiple lines, with each cause on its own line.
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
