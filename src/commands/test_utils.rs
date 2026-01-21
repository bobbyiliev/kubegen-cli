//! Test utilities for command modules
//!
//! Provides shared test infrastructure for commands that need to change
//! the current working directory.

#[cfg(test)]
use std::sync::Mutex;

/// Global mutex to serialize tests that change the current directory.
///
/// Tests that modify the current working directory must acquire this lock
/// to prevent race conditions when tests run in parallel.
#[cfg(test)]
pub static CWD_LOCK: Mutex<()> = Mutex::new(());
