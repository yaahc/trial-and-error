//! Experimental replacement for `Box<dyn Error>` that implements `Error`.
//!
//! This module introduces the `DynError` type, which owns an inner 
//! `Box<dyn Error>`. Most importantly, the `DynError` type implements the `Error`
//! trait, unlike a plain `Box<dyn Error>`. Another important distinction between
//! `DynError` and `Box<dyn Error>` is that `DynError` _doesn't_ implement 
//! `From<E: Error>`. 

use std::error::Error;
use std::fmt;

type BoxError = Box<dyn Error + Send + Sync + 'static>;

/// Owning type for a `BoxError`.
#[derive(Debug)]
pub struct DynError {
    /// The inner wrapped `BoxError`.
    error: BoxError,
}

impl fmt::Display for DynError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        self.error.fmt(f)
    }
}

/// This type _does_ implement error 🙌
impl Error for DynError {
    fn source(&self) -> Option<&(dyn Error + 'static)> {
        self.error.source()
    }
}

impl DynError {
    /// Create a new `DynError` from an input error.
    fn new<E>(error: E) -> Self
    where
        BoxError: From<E>,
    {
        let error = BoxError::from(error);

        // This logic is necessary for round tripping through `Result<T,
        // BoxError>`, as demonstrated in `fn thing_3()`
        //
        // This is effectively resolving the "overlap rule" issue with `Box<dyn
        // Error + ...>` at runtime by always boxing it and then checking if it
        // shouldn't have after-the-fact with `downcast`.
        //
        // Check if the erased error type is already the type we want
        match error.downcast::<DynError>() {
            // If it is use it directly
            Ok(box error) => error,
            // otherwise create a new `DynError` to wrap the type erased error
            Err(error) => DynError { error },
        }
    }
}

use std::ops::{ControlFlow, FromResidual, Try};
use std::process::Termination;

/// Result that always converts error types to an `DynError`.
pub enum DynResult<T> {
    /// The Ok variant of the `DynResult`.
    Ok(T),
    /// The Err variant of the `DynResult` containing a `DynError`.
    Err(DynError),
}

impl<T> Termination for DynResult<T> {
    /// Return an error code corresponding with the `DynResult`; 0 for success, 1 for failure.
    fn report(self) -> i32 {
        match self {
            DynResult::Ok(_) => 0,
            DynResult::Err(error) => {
                eprintln!("Error: {:?}", crate::Report::new(error));
                1
            }
        }
    }
}

// Implements `Try` on `DynResult` so that the `?` operator can be used on it
impl<T> Try for DynResult<T> {
    type Output = T;
    // `DynResult<!>` is a one-variant enum that can only ever hold an error variant
    // It can't possibly hold an Ok variant
    type Residual = DynResult<!>;

    fn from_output(value: T) -> Self {
        DynResult::Ok(value)
    }

    fn branch(self) -> ControlFlow<Self::Residual, T> {
        match self {
            DynResult::Ok(value) => ControlFlow::Continue(value),
            DynResult::Err(error) => ControlFlow::Break(DynResult::Err(error)),
        }
    }
}

impl<T, E> FromResidual<Result<!, E>> for DynResult<T>
where
    BoxError: From<E>,
{
    fn from_residual(inner: Result<!, E>) -> Self {
        let Err(error) = inner;
        let error = DynError::new(error);
        DynResult::Err(error)
    }
}

impl<T> FromResidual<DynResult<!>> for DynResult<T> {
    fn from_residual(residual: DynResult<!>) -> Self {
        let DynResult::Err(error) = residual;
        DynResult::Err(error)
    }
}

impl<T> FromResidual<DynResult<!>> for Result<T, BoxError> {
    fn from_residual(residual: DynResult<!>) -> Self {
        let DynResult::Err(error) = residual;
        let error = BoxError::from(error);
        Err(error)
    }
}
