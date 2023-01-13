# Scientisto
A light-weight Rust implementation of the [github/scientist](https://github.com/github/scientist) library used for careful refactoring of critical code paths.

`scientisto` ('scientist' in Esperanto) provides the `Experiment` struct used to define the conducted experiment and publishing utilities.

## About

The library aims to be as minimal as possible, pulling no external dependencies and using a bare minimum from the `std` library.

## Usage
`Experiment` struct represents a definition of an experimental code path with explicit implementations of the **control** and **experimental** execution blocks.

The experiment is guided by the configuration specified during the `Experiment` construction.

The experiment observations are published internally using the `publish` function.
```rust
use scientisto::Experiment;

let expected: i32 = 1;
let result = Experiment::new("Test")
    .control(|| expected)
    .experiment(|| expected + 1)
    .publish(|o: &crate::observation::Observation<i32, i32>| {
        tracing::info!("You can do any magic in the publisher")
     })
    .run();
```

## Limitations
- No defaults are provided for the `control` and `experiment` callbacks, they must be fully specified
- No `async` support