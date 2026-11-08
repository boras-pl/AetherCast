// Copyright © 2026 AetherCast Contributors
// SPDX-License-Identifier: MIT

//! Logging setup module.
//!
//! Initializes tracing subscriber with file and console output.

pub mod setup;

pub use setup::init_logging;
