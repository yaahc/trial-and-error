#![feature(backtrace)]

use std::fmt;
use std::error::Error;
use std::backtrace::Backtrace;

use trial_and_error::Report;

#[derive(Debug)]
struct SuperError<'a> {
    msg: &'a str,
    backtrace: Option<Backtrace>,
}

impl<'a> fmt::Display for SuperError<'a> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "SuperError is here: {}", self.msg)
    }
}

impl<'a> Error for SuperError<'a> {
    fn backtrace(&self) -> Option<&Backtrace> {
        match &self.backtrace {
            None => None,
            Some(bt) => Some(&bt),
        }
    }
}

fn main() {
    let error = SuperError {
        msg: "Huzzah!",
        backtrace: Some(Backtrace::capture()),
    };

    let report = Report::new(error)
        .pretty()
        .show_backtrace();

    println!("{}", report);
}
