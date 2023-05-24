/// `async` Experiment
/// Basic struct defining the conducted `async` experiment. Initialized using type definitions instead of
/// allocations. The `AsyncExperiment` is a consumable, once executed, it will consume the constituent
/// futures defined for the experiment.
///
/// The results of the `async` experiment, if run and awaited, are input into the publisher. The default
/// publisher is a `noop`, whereas a custom publisher can be used either as a passed function or
/// closure. Publisher can contain any logic, as long as it returns a `Unit` type.
///
/// # Operation
/// - decides whether or not to run the experiment block
/// - swallows and records exceptions raised in the try block when overriding raised
/// - publishes all this information
///
/// # Panics
/// If any of the constituent futures panics
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
/// use scientisto::{AsyncExperiment,Observation};
///
/// async fn production() -> f32 { 3.00 }
/// async fn alternative() -> f32 { 3.02 }
///
/// async_std::task::block_on(async {
///     AsyncExperiment::new("Using callback functions")
///         .control(production())
///         .experiment(alternative())
///         .publish(|o: &Observation<f32, f32>| assert!(!o.is_matching()))
///         .run().await;
/// })
/// ```
///
/// ## Using closures
/// ```rust
/// use scientisto::{AsyncExperiment,Observation};
/// use tracing::info;
///
/// async_std::task::block_on(async {
///     AsyncExperiment::new("Test")
///         .control(async { 3.0 })
///         .experiment(async { 3.0 })
///         .publish(|o: &Observation<f32, f32>| {
///             assert!(o.is_matching());
///             info!("Any logic, including side effects, can be here!")
///          })
///         .run().await;
/// })
/// ```
///
#[derive(Debug, Clone)]
pub struct AsyncExperiment {
    /// The name under which the experiment is registered.
    name: &'static str,
}

impl AsyncExperiment {
    pub fn new(name: &'static str) -> Self {
        if name.is_empty() {
            panic!("Experiment name cannot be empty");
        }

        Self { name }
    }

    pub fn name(&self) -> &'static str {
        self.name
    }

    pub fn control<T, F>(self, f: F) -> AsyncControlOnly<T, F>
    where
        F: std::future::Future<Output = T>,
    {
        AsyncControlOnly {
            name: self.name,
            control: f,
        }
    }
}

pub struct AsyncControlOnly<TC, FC>
where
    FC: std::future::Future<Output = TC>,
{
    name: &'static str,
    control: FC,
}

impl<TC, FC> AsyncControlOnly<TC, FC>
where
    FC: std::future::Future<Output = TC>,
{
    pub fn name(&self) -> &'static str {
        self.name
    }

    pub fn experiment<T, F>(
        self,
        f: F,
    ) -> AsyncCompleteExperiment<TC, FC, T, F, impl Fn(&crate::Observation<TC, T>)>
    where
        F: std::future::Future<Output = T>,
    {
        AsyncCompleteExperiment {
            name: self.name,
            control: self.control,
            experiment: f,
            publish: |_: &crate::Observation<TC, T>| {},
        }
    }
}

pub struct AsyncCompleteExperiment<TC, FC, TE, FE, FP>
where
    FC: std::future::Future<Output = TC>,
    FE: std::future::Future<Output = TE>,
{
    name: &'static str,
    control: FC,
    experiment: FE,
    publish: FP,
}

impl<TC, FC, TE, FE, FP> AsyncCompleteExperiment<TC, FC, TE, FE, FP>
where
    FC: std::future::Future<Output = TC>,
    FE: std::future::Future<Output = TE>,
{
    pub fn name(&self) -> &'static str {
        self.name
    }

    pub fn publish<F>(self, f: F) -> AsyncCompleteExperiment<TC, FC, TE, FE, F>
    where
        FC: std::future::Future<Output = TC>,
        FE: std::future::Future<Output = TE>,
        F: Fn(&crate::Observation<TC, TE>),
        TE: PartialEq<TC>,
    {
        AsyncCompleteExperiment::<TC, FC, TE, FE, F> {
            name: self.name,
            control: self.control,
            experiment: self.experiment,
            publish: f,
        }
    }

    pub async fn run(self) -> TC
    where
        FC: std::future::Future<Output = TC>,
        FE: std::future::Future<Output = TE>,
        FP: Fn(&crate::Observation<TC, TE>),
    {
        self.run_if(|| true).await
    }

    pub async fn run_if<P>(self, predicate: P) -> TC
    where
        FC: std::future::Future<Output = TC>,
        FE: std::future::Future<Output = TE>,
        FP: Fn(&crate::Observation<TC, TE>),
        P: Fn() -> bool,
    {
        let should_run_experiment = predicate();
        if should_run_experiment {
            let (control, experiment) = futures::join!(self.control, self.experiment);
            let observation = crate::Observation::<TC, TE> {
                control: Ok(control),
                experiment: Ok(experiment),
            };

            (self.publish)(&observation);

            observation.control.ok().unwrap()
        } else {
            self.control.await
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn experiment_should_derive_the_debug_trait() {
        let experiment = AsyncExperiment::new("empty experiment");

        assert_ne!(format!("{:?}", experiment), "");
    }

    #[test]
    #[should_panic]
    fn experiment_should_panic_on_empty_string_name() {
        std::panic::set_hook(Box::new(|_| {})); // hide traces from panic

        AsyncExperiment::new("");
    }

    #[test]
    fn async_experiment_should_return_name_if_it_is_valid() {
        let actual_name: &str = "Any ľšýžľš is OK";
        let experiment = AsyncExperiment::new(actual_name);

        assert_eq!(experiment.name(), actual_name);
    }

    #[test]
    fn async_experiment_should_return_name_with_control_specified() {
        let actual_name: &str = "Only control callback";
        let experiment = AsyncExperiment::new(actual_name).control(async { false });

        assert_eq!(experiment.name(), actual_name);
    }

    #[test]
    fn async_experiment_should_return_name_if_control_and_experiment_are_fully_specified() {
        let name: &str = "Only control callback";
        let experiment = AsyncExperiment::new(name)
            .control(async { 1 })
            .experiment(async { 1 });

        assert_eq!(experiment.name(), name);
    }

    #[async_std::test]
    async fn async_experiment_should_always_return_the_control_value() {
        let expected = 1;
        let actual = AsyncExperiment::new("Test")
            .control(async { expected })
            .experiment(async { expected })
            .run()
            .await;

        assert_eq!(actual, expected);
    }

    #[async_std::test]
    async fn async_experiment_should_not_run_the_experiment_if_conditioned_not_to() {
        let expected = 1;
        let actual = AsyncExperiment::new("Test")
            .control(async { expected })
            .experiment(async { expected })
            .publish(|_o: &crate::Observation<i32, i32>| {})
            .run_if(|| false)
            .await;

        assert_eq!(actual, expected);
    }

    #[async_std::test]
    async fn async_experiment_should_publish_the_results_when_publish_method_is_specified() {
        let expected = 1;
        AsyncExperiment::new("Test")
            .control(async { expected })
            .experiment(async { expected })
            .publish(|o: &crate::Observation<i32, i32>| assert!(o.is_matching()))
            .run()
            .await;
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

    #[async_std::test]
    async fn async_experiment_should_work_with_different_return_types_if_they_are_comparable() {
        let expected: i32 = 1;
        let expected_as_i64 = TestI64 {
            value: expected as i64,
        };

        assert!(expected_as_i64 == expected_as_i64); // implements PartialEq

        AsyncExperiment::new("Test")
            .control(async { expected })
            .experiment(async { expected_as_i64 })
            .publish(|o: &crate::Observation<i32, TestI64>| assert!(o.is_matching()))
            .run()
            .await;
    }

    #[async_std::test]
    #[should_panic]
    async fn async_experiment_should_panic_if_control_panics() {
        std::panic::set_hook(Box::new(|_| {})); // hide traces from panic

        let expected: i32 = 1;
        AsyncExperiment::new("Test")
            .control(async { panic!("Oops") })
            .experiment(async { expected })
            .publish(|_o: &crate::Observation<i32, i32>| {})
            .run()
            .await;
    }

    #[async_std::test]
    async fn async_experiment_should_return_control_value_if_the_experiment_value_is_different() {
        let expected: i32 = 1;
        AsyncExperiment::new("Test")
            .control(async { expected })
            .experiment(async { expected + 1 })
            .publish(|o: &crate::Observation<i32, i32>| assert!(!o.is_matching()))
            .run()
            .await;
    }
}
