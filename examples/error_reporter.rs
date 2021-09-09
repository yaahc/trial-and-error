use std::fmt;
use std::error::Error;

use trial_and_error::Report;

#[derive(Debug)]
struct SuperError;

impl fmt::Display for SuperError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "SuperError is here!")
    }
}

impl Error for SuperError {}

fn main() {
    let report = Report::new(SuperError).pretty();

    println!("{}", report);
}
