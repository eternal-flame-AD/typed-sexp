//! This module contains typed wrappers for R's SEXP type.

/// A wrapper around R matrices.
pub mod matrix;

/// Wrappers around simple R vectors.
pub mod vector;

/// A wrapper around R's language objects.
pub mod lang;

/// A wrapper around R's environments.
pub mod env;

/// A wrapper around R's functions.
pub mod function;

/// A wrapper around R's external pointers.
pub mod ptr;
