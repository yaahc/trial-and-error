//! Experimental replacement for `Box<dyn Error>` that implements `Error`.
//!
//! This module introduces the `DynError` type, which owns an inner `Box<dyn Error>`. Most
//! importantly, `DynError` _does_ implement the `Error` trait, unlike `Box<dyn Error>`. As a
//! result, `DynError` is more conveniently compatible with the rest of the error handling
//! ecosystem, and behaves just like any other error type does.
//!
//! # Background
//!
//! The short answer as to why `Box<dyn Error>` doesn't implement `Error` is because there
//! exists a blanket implementation of the `Error` trait for `Box<T>`, more specifically:
//! `impl<T: Error + Sized> Error for Box<T>`. Crucially, any type `T` must be sized. However,
//! when `T` is a `dyn Error` trait object, it is _not_ sized. This constraint could be loosened by
//! altering the implementation to not require that `T` be `Sized`, such that it becomes
//! `impl<T: Error + ?Sized> Error for Box<T>`, however, this altered implementation causes overlap
//! between the `From` trait's `impl<T> From<T> for T` blanket implementation and Box's `impl<'a,
//! E: Error + 'a> From<E> for Box<dyn Error + 'a>`.
//!
//! For more context on why `Box<dyn Error>` doesn't implement the `Error` trait, see
//! https://stackoverflow.com/questions/65151237/why-doesnt-boxdyn-error-implement-error.
//!
//! `DynError` circumvents this overlap by being paired with a corresponding `DynResult` type that
//! implements its own set of `FromResidual` impls (these exist so that `DynResult` works the same
//! way with the `?` operator as `Result`). However, as a result, `DynError`s can only be
//! constructed with `?` from arbitrary error types when paired with `DynResult`. Using a
//! `Result<T, DynError>` will require manual conversion of error types due to it missing the
//! `From` impl that is present on `Box<dyn Error>`.
//!
//! # Examples
//!
//! `DynResult` is not quite a drop-in replacement for `Box<dyn Error>`. Some noticeable
//! differences include the aforementioned requirement that any time we have a function that might
//! result in a `DynResult`, that function must use the corresponding `DynResult` type as its
//! return type.
//!
//! ```rust
//! use trial_and_error::DynResult;
//!
//! fn main() -> DynResult<()> {
//!     let _parsed = "4".parse::<u32>()?;
//!
//!     DynResult::Ok(())
//! }
//! ```
//!
//! Additionally, since `DynError` wraps a `BoxError` type, which is an alias for `Box<dyn Error +
//! Send + Sync + 'static>`, any error handling API that requires type erased non-thread-safe
//! errors would not be able to make use of `DynError`.

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

/// This type _does_ implement `Error` 🙌
impl Error for DynError {
    fn source(&self) -> Option<&(dyn Error + 'static)> {
        self.error.source()
    }
}

impl DynError {
    /// Create a new `DynError` from an input error.
    fn new<E>(error: E) -> Self
    where
        E: Error + Send + Sync + 'static,
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

// Given a `Result::Err(E)`, convert it to a `DynResult::Err(E)`
impl<T, E> FromResidual<Result<!, E>> for DynResult<T>
where
    E: Error + Send + Sync + 'static,
{
    fn from_residual(inner: Result<!, E>) -> Self {
        let Err(error) = inner;
        let error = DynError::new(error);
        DynResult::Err(error)
    }
}

// Given a `DynResult` emitted by a `?`, convert it to a `DynResult::Err(E)`
impl<T> FromResidual<DynResult<!>> for DynResult<T> {
    fn from_residual(residual: DynResult<!>) -> Self {
        let DynResult::Err(error) = residual;
        DynResult::Err(error)
    }
}

// Given a `DynResult` emitted by a `?`, convert it to a `Result::Err(E)`
impl<T> FromResidual<DynResult<!>> for Result<T, BoxError> {
    fn from_residual(residual: DynResult<!>) -> Self {
        let DynResult::Err(error) = residual;
        let error = BoxError::from(error);
        Err(error)
    }
}