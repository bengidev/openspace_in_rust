//! Git feature domain layer.
//!
//! Pure value types: diff hunks, commit metadata, branch refs.
//! Implementation details (libgit2 calls, command-line shelling)
//! live in the infrastructure layer.

pub mod git_diff;
