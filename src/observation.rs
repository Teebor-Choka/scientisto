use std::thread::Result;
use std::time::{Duration, Instant};

/// Measurement
/// Individual measurement for the executed functionality.
#[derive(Debug)]
pub struct Measurement<T> {
    pub result: Result<T>,
    pub duration: Duration,
}

impl Measurement<()> {
    pub fn new<T>(result: Result<T>, duration: Duration) -> Measurement<T> {
        Measurement { result, duration }
    }
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
        result: std::panic::catch_unwind(std::panic::AssertUnwindSafe(function)),
        duration: timer.elapsed(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn measurement_should_derive_the_debug_trait() {
        let measurement = Measurement::new::<i32>(Result::Ok(1), Duration::from_secs(1));

        assert_ne!(format!("{:?}", measurement), "");
    }

    #[test]
    fn observation_should_derive_the_debug_trait() {
        let observation = Observation::<i32, i32> {
            control: Measurement::new::<i32>(Result::Ok(1), Duration::from_secs(1)),
            experiment: Measurement::new::<i32>(Result::Ok(1), Duration::from_secs(1)),
        };

        assert_ne!(format!("{:?}", observation), "");
    }

    #[test]
    fn observation_should_indicate_matching_when_comparable_types_have_matching_values() {
        let observation = Observation::<i32, i32> {
            control: Measurement::new::<i32>(Result::Ok(1), Duration::from_secs(1)),
            experiment: Measurement::new::<i32>(Result::Ok(1), Duration::from_secs(1)),
        };

        assert!(observation.is_matching())
    }

    #[test]
    fn observation_result_matchability_should_not_depend_on_the_durations() {
        let observation = Observation::<i32, i32> {
            control: Measurement::new::<i32>(Result::Ok(1), Duration::from_secs(1)),
            experiment: Measurement::new::<i32>(Result::Ok(1), Duration::from_secs(2)),
        };

        assert!(observation.is_matching())
    }

    #[test]
    fn observation_should_indicate_non_matching_when_comparable_types_have_non_matching_values() {
        let observation = Observation::<i32, i32> {
            control: Measurement::new::<i32>(Result::Ok(1), Duration::from_secs(1)),
            experiment: Measurement::new::<i32>(Result::Ok(2), Duration::from_secs(1)),
        };

        assert!(!observation.is_matching())
    }

    #[test]
    fn observation_should_indicate_non_matching_when_non_matching_result_values_are_measured() {
        let observation = Observation::<i32, i32> {
            control: Measurement::new::<i32>(Result::Ok(1), Duration::from_secs(1)),
            experiment: Measurement::new::<i32>(
                Result::Err(Box::new("Error")),
                Duration::from_secs(1),
            ),
        };

        assert!(!observation.is_matching())
    }
}
