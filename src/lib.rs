//! `Experiment` struct represents a definition of the **control** and **experimental** execution block code paths.
//!
//! The experiment is guided by the configuration specified during the `Experiment` construction and result
//! observations are published internally using the `publish` functionExperiment` struct represents a
//! definition of an experimental code path with explicit.
//!
//! # Example usage
//! ```rust
//! use scientisto::*;
//! use tracing;
//!
//! let expected: i32 = 1;
//! let result = Experiment::new("Test")
//!     .control(|| expected)
//!     .experiment(|| expected + 1)
//!     .publish(|o: &scientisto::Observation<i32, i32>| {
//!         tracing::info!("You can do any magic in the publisher")
//!      })
//!     .run();
//! ```

pub mod experiment;
pub mod observation;

pub use experiment::Experiment;
pub use observation::Observation;
