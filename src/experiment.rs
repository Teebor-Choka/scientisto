use std::marker::PhantomData;

/// Experiment
/// Basic struct defining the conducted experiment. Initialized using type definitions instead of
/// allocations. The `Experiment` is a consumable, once executed, it will consume the constituent
/// functions defined for the experiment.
///
/// The results of the experiment, if run, are input into the publisher. The default
/// publisher is a `noop`, whereas a custom publisher can be used either as a passed function or
/// closure. Publisher can contain any logic, as long as it returns a `Unit` type.
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
///     .publish(|o: &scientisto::observation::Observation<f32, f32>| assert!(!o.is_matching()))
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
///     .publish(|o: &scientisto::observation::Observation<f32, f32>| {
///         assert!(o.is_matching());
///         tracing::info!("Any logic, including side effects, can be here!")
///      })
///     .run();
/// ```
///
#[derive(Debug)]
pub struct Experiment<T, TE, C, E, P>
where
    T: PartialEq,
    TE: PartialEq<T>,
{
    /// The name under which the experiment is registered.
    name: &'static str,
    /// Phantom data used to allow extracting the type of control callback return value
    phantom_return_type_control: PhantomData<T>,
    /// Phantom data used to allow extracting the type of experimental callback return value
    phantom_return_type_experiment: PhantomData<TE>,
    /// Control callback function
    control_cb: C,
    /// Experimental callback function
    experiment_cb: E,
    /// Publish callback function
    publish_cb: P,
}

impl Experiment<bool, bool, (), (), ()> {
    pub fn new(name: &'static str) -> Experiment<bool, bool, (), (), ()> {
        if name.is_empty() {
            panic!("Experiment name cannot be empty");
        }

        Experiment {
            name,
            phantom_return_type_control: PhantomData,
            phantom_return_type_experiment: PhantomData,
            control_cb: (),
            experiment_cb: (),
            publish_cb: (),
        }
    }
}

impl<T> Experiment<T, T, (), (), ()>
where
    T: PartialEq,
{
    pub fn control<NT, NC>(
        self,
        control_cb: NC,
    ) -> Experiment<NT, NT, NC, (), impl Fn(&crate::observation::Observation<NT, NT>)>
    where
        NT: PartialEq,
        NC: Fn() -> NT + std::panic::UnwindSafe,
    {
        let dummy_publish = |_l: &crate::observation::Observation<NT, NT>| {};
        Experiment {
            name: self.name,
            phantom_return_type_control: PhantomData,
            phantom_return_type_experiment: PhantomData,
            control_cb,
            experiment_cb: self.experiment_cb,
            publish_cb: dummy_publish,
        }
    }
}

impl<T, TE, C, P> Experiment<T, TE, C, (), P>
where
    T: PartialEq,
    TE: PartialEq<T>,
{
    pub fn experiment<NTE, NE>(
        self,
        experiment_cb: NE,
    ) -> Experiment<T, NTE, C, NE, impl Fn(&crate::observation::Observation<T, NTE>)>
    where
        C: Fn() -> T + std::panic::UnwindSafe,
        NE: Fn() -> NTE + std::panic::UnwindSafe,
        NTE: PartialEq<T>,
    {
        let dummy_publish = |_l: &crate::observation::Observation<T, NTE>| {};
        Experiment {
            name: self.name,
            phantom_return_type_control: PhantomData,
            phantom_return_type_experiment: PhantomData,
            control_cb: self.control_cb,
            experiment_cb,
            publish_cb: dummy_publish,
        }
    }
}

impl<T, TE, C, E, P> Experiment<T, TE, C, E, P>
where
    T: PartialEq,
    TE: PartialEq<T>,
{
    pub fn name(&self) -> &str {
        self.name
    }

    pub fn publish<NP>(self, publish_cb: NP) -> Experiment<T, TE, C, E, NP>
    where
        C: Fn() -> T,
        E: Fn() -> TE,
        NP: Fn(&crate::observation::Observation<T, TE>),
    {
        Experiment {
            name: self.name,
            phantom_return_type_control: PhantomData,
            phantom_return_type_experiment: PhantomData,
            control_cb: self.control_cb,
            experiment_cb: self.experiment_cb,
            publish_cb,
        }
    }

    pub fn run(self) -> T
    where
        C: Fn() -> T + std::panic::UnwindSafe,
        E: Fn() -> TE + std::panic::UnwindSafe,
        P: Fn(&crate::observation::Observation<T, TE>),
    {
        self.run_if(|| true)
    }

    pub fn run_if<Predicate>(self, condition: Predicate) -> T
    where
        C: Fn() -> T + std::panic::UnwindSafe,
        E: Fn() -> TE + std::panic::UnwindSafe,
        P: Fn(&crate::observation::Observation<T, TE>),
        Predicate: Fn() -> bool,
    {
        // It decides whether or not to run the try block,
        // Measures the durations of all behaviors as std::time::Duration,
        // Swallow and record exceptions raised in the try block when overriding raised, and
        // Publishes all this information.

        if condition() {
            let control = crate::observation::execute_with_timer::<C, T>(self.control_cb);
            let experiment = crate::observation::execute_with_timer::<E, TE>(self.experiment_cb);

            let observation = crate::observation::Observation::<T, TE> {
                control,
                experiment,
            };

            (self.publish_cb)(&observation);

            match observation.control.result {
                Ok(result) => result,
                Err(e) => std::panic::resume_unwind(e),
            }
        } else {
            (self.control_cb)()
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    #[should_panic]
    fn experiment_should_panic_on_empty_string_name() {
        Experiment::new("");
    }

    #[test]
    fn experiment_should_return_name_if_it_is_valid() {
        let actual_name: &str = "Any ľšýžľš is OK";
        let experiment = Experiment::new(actual_name);

        assert_eq!(experiment.name(), actual_name);
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
            .publish(|o: &crate::observation::Observation<i32, i32>| assert!(o.is_matching()))
            .run();
    }

    #[derive(PartialEq, Debug, Copy, Clone)]
    struct TestI32 {
        value: i32,
    }

    #[derive(PartialEq, Debug, Copy, Clone)]
    struct TestI64 {
        value: i64,
    }

    impl PartialEq<TestI32> for TestI64 {
        fn eq(&self, other: &TestI32) -> bool {
            self.value as i32 == other.value
        }
    }

    impl PartialEq<TestI64> for TestI32 {
        fn eq(&self, other: &TestI64) -> bool {
            self.value == other.value as i32
        }
    }

    #[test]
    fn experiment_should_work_with_different_return_types_if_they_are_comparable() {
        let expected = TestI32 { value: 1 };
        Experiment::new("Test")
            .control(|| expected)
            .experiment(|| {
                (TestI64 {
                    value: expected.value as i64,
                })
            })
            .publish(|o: &crate::observation::Observation<TestI32, TestI64>| {
                assert!(o.is_matching())
            })
            .run();
    }

    #[test]
    #[should_panic]
    fn experiment_should_panic_if_control_panics() {
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
            .publish(|o: &crate::observation::Observation<i32, i32>| assert!(!o.is_matching()))
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
}
