use std::marker::PhantomData;
use std::panic::{catch_unwind, AssertUnwindSafe};

/// Experiment
/// Basic struct defining the conducted experiment. Initialized using type definitions instead of
/// allocations. The `Experiment` is a consumable, once executed, it will consume the constituent
/// functions defined for the experiment.
///
/// The results of the experiment, if run, are input into the publisher. The default
/// publisher is a `noop`, whereas a custom publisher can be used either as a passed function or
/// closure. Publisher can contain any logic, as long as it returns a `Unit` type.
///
/// # Operation
/// - decides whether or not to run the experiment block
/// - measures the durations of all behaviors as std::time::Duration
/// - swallows and records exceptions raised in the try block when overriding raised
/// - publishes all this information
///
/// # Panics
/// Panics if the **control** function panics using the `std::panic::resume_unwind`.
///
/// # Errors
/// None
///
/// # Safety
/// No `unsafe` code is executed outside the `std` usage.
///
/// # Examples
/// ## Using function callbacks
/// ```rust
/// use scientisto::Experiment;
///
/// fn production() -> f32 { 3.00 }
/// fn alternative() -> f32 { 3.02 }
///
/// Experiment::new("Using callback functions")
///     .control(production)
///     .experiment(alternative)
///     .publish(|o: &scientisto::Observation<f32, f32>| assert!(!o.is_matching()))
///     .run();
/// ```
///
/// ## Using closures
/// ```rust
/// use scientisto::Experiment;
/// use tracing::info;
///
/// Experiment::new("Test")
///     .control(|| -> f32 { 3.00 })
///     .experiment(|| -> f32 { 3.00 })
///     .publish(|o: &scientisto::Observation<f32, f32>| {
///         assert!(o.is_matching());
///         info!("Any logic, including side effects, can be here!")
///      })
///     .run();
/// ```
///

struct Executable<T, F>
where
    F: Fn() -> T,
{
    phantom_return_type: PhantomData<T>,
    pub f: F,
}

impl<T, F> Executable<T, F>
where
    F: Fn() -> T,
{
    pub fn new(f: F) -> Self {
        Self {
            phantom_return_type: Default::default(),
            f,
        }
    }
}

#[derive(Debug, Clone)]
pub struct Experiment {
    /// The name under which the experiment is registered.
    name: &'static str,
}

impl Experiment {
    pub fn new(name: &'static str) -> Self {
        if name.is_empty() {
            panic!("Experiment name cannot be empty");
        }

        Self { name: name }
    }

    pub fn name(&self) -> &'static str {
        self.name
    }

    pub fn control<T, F>(self, f: F) -> ControlOnly<T, F>
    where
        F: Fn() -> T + std::panic::UnwindSafe,
    {
        ControlOnly {
            name: self.name,
            control: Executable::<T, F>::new(f),
        }
    }
}

pub struct ControlOnly<TC, FC>
where
    FC: Fn() -> TC + std::panic::UnwindSafe,
{
    name: &'static str,
    control: Executable<TC, FC>,
}

impl<TC, FC> ControlOnly<TC, FC>
where
    FC: Fn() -> TC + std::panic::UnwindSafe,
{
    pub fn name(&self) -> &'static str {
        self.name
    }

    pub fn experiment<T, F>(
        self,
        f: F,
    ) -> CompleteExperiment<TC, FC, T, F, impl Fn(&crate::Observation<TC, T>)>
    where
        F: Fn() -> T + std::panic::UnwindSafe,
    {
        CompleteExperiment {
            name: self.name,
            control: self.control,
            experiment: Executable::<T, F>::new(f),
            publish: |_: &crate::Observation<TC, T>| {},
        }
    }
}

pub struct CompleteExperiment<TC, FC, TE, FE, FP>
where
    FC: Fn() -> TC + std::panic::UnwindSafe,
    FE: Fn() -> TE + std::panic::UnwindSafe,
{
    name: &'static str,
    control: Executable<TC, FC>,
    experiment: Executable<TE, FE>,
    publish: FP,
}

impl<TC, FC, TE, FE, FP> CompleteExperiment<TC, FC, TE, FE, FP>
where
    FC: Fn() -> TC + std::panic::UnwindSafe,
    FE: Fn() -> TE + std::panic::UnwindSafe,
{
    pub fn name(&self) -> &'static str {
        self.name
    }

    pub fn publish<F>(self, f: F) -> CompleteExperiment<TC, FC, TE, FE, F>
    where
        TE: PartialEq<TC>,
        F: Fn(&crate::Observation<TC, TE>),
    {
        CompleteExperiment::<TC, FC, TE, FE, F> {
            name: self.name,
            control: self.control,
            experiment: self.experiment,
            publish: f,
        }
    }

    pub fn run(&self) -> TC
    where
        TE: PartialEq<TC>,
        FP: Fn(&crate::Observation<TC, TE>),
    {
        self.run_if(|| true)
    }

    pub fn run_if<P>(&self, predicate: P) -> TC
    where
        TE: PartialEq<TC>,
        FP: Fn(&crate::Observation<TC, TE>),
        P: Fn() -> bool,
    {
        if predicate() {
            let observation = crate::Observation::<TC, TE> {
                control: catch_unwind(AssertUnwindSafe(&self.control.f)),
                experiment: catch_unwind(AssertUnwindSafe(&self.experiment.f)),
            };

            (self.publish)(&observation);

            match observation.control {
                Ok(result) => result,
                Err(e) => std::panic::resume_unwind(e),
            }
        } else {
            (self.control.f)()
        }
    }

    pub fn async_publish<F, U, V>(self, f: F) -> CompleteExperiment<TC, FC, TE, FE, F>
    where
        TC: std::future::Future<Output = U>,
        TE: std::future::Future<Output = V>,
        F: Fn(&crate::Observation<U, V>),
        V: PartialEq<U>,
    {
        CompleteExperiment::<TC, FC, TE, FE, F> {
            name: self.name,
            control: self.control,
            experiment: self.experiment,
            publish: f,
        }
    }

    pub fn async_run<U, V>(self) -> impl std::future::Future<Output = U>
    where
        TC: std::future::Future<Output = U>,
        TE: std::future::Future<Output = V>,
        FP: Fn(&crate::Observation<U, V>),
    {
        self.async_run_if(|| true)
    }

    pub fn async_run_if<P, U, V>(self, predicate: P) -> impl std::future::Future<Output = U>
    where
        TC: std::future::Future<Output = U>,
        TE: std::future::Future<Output = V>,
        FP: Fn(&crate::Observation<U, V>),
        P: Fn() -> bool,
    {
        let should_run_experiment = predicate();
        async move {
            if should_run_experiment {
                let (control, experiment) =
                    futures::join!((&self.control.f)(), (&self.experiment.f)());
                let observation = crate::Observation::<U, V> {
                    control: Ok(control),
                    experiment: Ok(experiment),
                };

                (self.publish)(&observation);

                match observation.control {
                    Ok(result) => result,
                    Err(e) => std::panic::resume_unwind(e),
                }
            } else {
                (self.control.f)().await
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn experiment_should_derive_the_debug_trait() {
        let experiment = Experiment::new("empty experiment");

        assert_ne!(format!("{:?}", experiment), "");
    }

    #[test]
    #[should_panic]
    fn experiment_should_panic_on_empty_string_name() {
        std::panic::set_hook(Box::new(|_| {})); // hide traces from panic

        Experiment::new("");
    }

    #[test]
    fn experiment_should_return_name_if_it_is_valid() {
        let actual_name: &str = "Any ľšýžľš is OK";
        let experiment = Experiment::new(actual_name);

        assert_eq!(experiment.name(), actual_name);
    }

    #[test]
    fn experiment_should_return_name_the_control_object() {
        let actual_name: &str = "Only control callback";
        let experiment = Experiment::new(actual_name).control(|| false);

        assert_eq!(experiment.name(), actual_name);
    }

    #[test]
    fn experiment_should_return_name_if_control_and_experiment_are_fully_specified() {
        let name: &str = "Only control callback";
        let experiment = Experiment::new(name).control(|| 1).experiment(|| 1);

        assert_eq!(experiment.name(), name);
    }

    #[test]
    fn experiment_should_always_return_the_control_value() {
        let expected = 1;
        let actual = Experiment::new("Test")
            .control(|| expected)
            .experiment(|| expected)
            .run();

        assert_eq!(actual, expected);
    }

    #[test]
    fn experiment_should_not_run_the_experiment_if_conditioned_not_to() {
        let expected = 1;
        let actual = Experiment::new("Test")
            .control(|| expected)
            .experiment(|| expected)
            .run_if(|| false);

        assert_eq!(actual, expected);
    }

    #[test]
    fn experiment_should_publish_the_results_when_publish_method_is_specified() {
        let expected = 1;
        Experiment::new("Test")
            .control(|| expected)
            .experiment(|| expected)
            .publish(|o: &crate::Observation<i32, i32>| assert!(o.is_matching()))
            .run();
    }

    #[derive(PartialEq, Copy, Clone)]
    struct TestI64 {
        value: i64,
    }

    impl PartialEq<i32> for TestI64 {
        fn eq(&self, other: &i32) -> bool {
            self.value as i32 == *other
        }
    }

    #[test]
    fn experiment_should_work_with_different_return_types_if_they_are_comparable() {
        let expected: i32 = 1;
        let expected_as_i64 = TestI64 {
            value: expected as i64,
        };

        assert!(expected_as_i64 == expected_as_i64); // implements PartialEq

        Experiment::new("Test")
            .control(move || expected)
            .experiment(move || expected_as_i64)
            .publish(|o: &crate::Observation<i32, TestI64>| assert!(o.is_matching()))
            .run();
    }

    #[test]
    #[should_panic]
    fn experiment_should_panic_if_control_panics() {
        std::panic::set_hook(Box::new(|_| {})); // hide traces from panic

        let expected: i32 = 1;
        Experiment::new("Test")
            .control(|| -> i32 { panic!("Oops") })
            .experiment(|| expected)
            .run();
    }

    #[test]
    fn experiment_should_return_control_value_if_the_experiment_value_is_different() {
        let expected: i32 = 1;
        Experiment::new("Test")
            .control(|| expected)
            .experiment(|| expected + 1)
            .publish(|o: &crate::Observation<i32, i32>| assert!(!o.is_matching()))
            .run();
    }

    #[test]
    fn experiment_should_return_control_value_if_the_experiment_panics() {
        let expected: i32 = 1;
        Experiment::new("Test")
            .control(|| expected)
            .experiment(|| -> i32 { panic!("Yikes") })
            .run();
    }

    #[async_std::test]
    async fn experiment_should_work_async_functions_returning_comparable_types() {
        let expected: i32 = 1;
        let expected_as_i64 = TestI64 {
            value: expected as i64,
        };

        assert!(expected_as_i64 == expected_as_i64); // implements PartialEq

        Experiment::new("Test")
            .control(move || async move { expected })
            .experiment(move || async move { expected_as_i64 })
            .async_publish(move |o: &crate::Observation<i32, TestI64>| assert!(o.is_matching()))
            .async_run()
            .await;
    }
}
