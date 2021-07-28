# Trial and Error

An experimental crate for proof-of-concept proposals from the Error Handling Project Group

This crate contains the following experimental modules:

1. An alternative to `Box<dyn Error + ...>` that implements the `Error` trait.
2. An error reporter that wraps an error and handles iterating over sources and formatting of error reports.
