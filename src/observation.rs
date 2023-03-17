use std::thread::Result;

/// Observation
///
/// Observation aggregating the measurements collected during execution of the control and
/// experimental functionality.
#[derive(Debug)]
pub struct Observation<T, TE>
where
    TE: PartialEq<T>,
{
    pub control: Result<T>,
    pub experiment: Result<TE>,
}

impl<T, TE> Observation<T, TE>
where
    TE: PartialEq<T>,
{
    /// Verify whether the control and experiment output a comparably equal or matching value.
    pub fn is_matching(&self) -> bool {
        match (&self.experiment, &self.control) {
            (Ok(a), Ok(b)) => a == b,
            _ => false,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn observation_should_derive_the_debug_trait() {
        let observation = Observation::<i32, i32> {
            control: Result::Ok(1),
            experiment: Result::Ok(1),
        };

        assert_ne!(format!("{:?}", observation), "");
    }

    #[test]
    fn observation_should_indicate_matching_when_comparable_types_have_matching_values() {
        let observation = Observation::<i32, i32> {
            control: Result::Ok(1),
            experiment: Result::Ok(1),
        };

        assert!(observation.is_matching())
    }

    #[test]
    fn observation_should_indicate_non_matching_when_comparable_types_have_non_matching_values() {
        let observation = Observation::<i32, i32> {
            control: Result::Ok(1),
            experiment: Result::Ok(2),
        };

        assert!(!observation.is_matching())
    }

    #[test]
    fn observation_should_indicate_non_matching_when_non_matching_result_values_are_measured() {
        let observation = Observation::<i32, i32> {
            control: Result::Ok(1),
            experiment: Result::Err(Box::new("Error")),
        };

        assert!(!observation.is_matching())
    }
}
