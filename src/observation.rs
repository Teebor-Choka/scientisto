use std::thread::Result;
use std::time::{Duration, Instant};

/// Measurement
/// Individual measurement for the executed functionality.
#[derive(Debug)]
pub struct Measurement<T> {
    pub result: Result<T>,
    pub duration: Duration,
}

/// Observation
/// Observation aggregating the measurements collected during execution of the control and experimental
/// functionality.
#[derive(Debug)]
pub struct Observation<T, TE>
where
    T: PartialEq,
    TE: PartialEq<T>,
{
    pub control: Measurement<T>,
    pub experiment: Measurement<TE>,
}

impl<T, TE> Observation<T, TE>
where
    T: PartialEq,
    TE: PartialEq<T>,
{
    /// Verify whether the control and experiment output a comparably equal or matching value.
    pub fn is_matching(&self) -> bool {
        match (&self.experiment.result, &self.control.result) {
            (Ok(a), Ok(b)) => a == b,
            _ => false,
        }
    }
}

/// Execute a function and measure the execution time.
pub fn execute_with_timer<F, U>(function: F) -> Measurement<U>
where
    F: Fn() -> U + std::panic::UnwindSafe,
{
    let timer = Instant::now();
    Measurement::<U> {
        result: std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| (function)())),
        duration: timer.elapsed(),
    }
}
