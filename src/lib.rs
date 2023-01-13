//! `Experiment` struct represents a definition of an experimental code path with explicit
//! implementations of the **control** and **experimental** execution blocks.
//!
//! The experiment is guided by the configuration specified during the `Experiment` construction.
//!
//! The experiment observations are published internally using the `publish` function.
//!
//! # Example usage
//! ```rust
//! use scientisto::Experiment;
//!
//! let expected: i32 = 1;
//! let result = Experiment::new("Test")
//!     .control(|| expected)
//!     .experiment(|| expected + 1)
//!     .publish(|o: &crate::observation::Observation<i32, i32>| {
//!         tracing::info!("You can do any magic in the publisher")
//!      })
//!     .run();
//! ```

pub mod experiment;
pub mod observation;

pub use experiment::Experiment;
pub use observation::Observation;
