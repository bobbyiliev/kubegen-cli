//! Command implementations for kubegen CLI
//!
//! This module contains the actual implementation of each CLI command.

mod crd;
mod new;

pub use crd::execute_add_crd;
pub use new::execute_new;
