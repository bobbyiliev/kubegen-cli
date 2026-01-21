//! Command implementations for kubegen CLI
//!
//! This module contains the actual implementation of each CLI command.

mod crd;
mod metrics;
mod new;
#[cfg(test)]
mod test_utils;
mod webhook;

pub use crd::execute_add_crd;
pub use metrics::execute_add_metrics;
pub use new::execute_new;
pub use webhook::execute_add_webhook;
