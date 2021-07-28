//! An experimental crate for proposals from the error handling project group.
//!
//! This crate currently contains two experiments, each in their own module.
//!
//! 1. An alternative to `Box<dyn Error + ...>` that implements `Error`.
//! 2. An error reporter that wraps an error and handles iterating over sources
//!    and formatting a full error report.
//!
#![feature(try_trait_v2)]
#![feature(termination_trait_lib)]
#![feature(never_type)]
#![feature(control_flow_enum)]
#![feature(box_patterns)]
#![feature(exhaustive_patterns)]
#![feature(backtrace)]
#![feature(error_iter)]

pub mod boxerror_replacement;
pub mod error_reporter;

pub use boxerror_replacement::{DynError, DynResult};
pub use error_reporter::Report;
