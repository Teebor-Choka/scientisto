//! `Experiment` struct represents a definition of the **control** and **experimental** execution block code paths.
//!
//! The experiment is guided by the configuration specified during the `Experiment` construction and result
//! observations are published internally using the `publish` functionExperiment` struct represents a
//! definition of an experimental code path with explicit.
//!
//! # Example usage
//! Non `async` code:
//! ```rust
//! use scientisto::*;
//! use tracing;
//!
//! let expected: i32 = 1;
//! let result = Experiment::new("Test")
//!     .control(|| expected)
//!     .experiment(|| expected + 1)
//!     .publish(|o: &Observation<i32, i32>| {
//!         tracing::info!("You can do any magic in the publisher")
//!      })
//!     .run();
//! ```
//!
//! `async` code:
//! ```rust
//! use scientisto::*;
//! use tracing;
//!
//! let expected: i32 = 1;
//! futures::executor::block_on(async {
//!     let result = AsyncExperiment::new("Test")
//!         .control(async { expected })
//!         .experiment(async { expected + 1 } )
//!         .publish(|o: &Observation<i32, i32>| {
//!             tracing::info!("You can do any magic in the publisher")
//!         })
//!         .run().await;
//! })
//! ```

#[cfg(feature = "async")]
pub mod async_experiment;

pub mod observation;

#[cfg(feature = "sync")]
pub mod sync_experiment;

pub use observation::Observation;

#[cfg(feature = "async")]
pub use async_experiment::AsyncExperiment;

#[cfg(feature = "sync")]
pub use sync_experiment::Experiment;
