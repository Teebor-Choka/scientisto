# Scientisto

[![Crates.io](https://img.shields.io/crates/v/scientisto.svg)](https://crates.io/crates/scientisto) [![MIT licensed](https://img.shields.io/badge/license-MIT-blue.svg)](https://github.com/Teebor-Choka/scientisto/blob/main/LICENSE) [![Publish](https://github.com/Teebor-Choka/scientisto/actions/workflows/publish.yaml/badge.svg)](https://github.com/Teebor-Choka/scientisto/actions/workflows/publish.yaml) [![codecov](https://codecov.io/gh/Teebor-Choka/scientisto/branch/main/graph/badge.svg?token=NHJU2F94UZ)](https://codecov.io/gh/Teebor-Choka/scientisto)

`scientisto` is a light-weight Rust implementation of the [github/scientist](https://github.com/github/scientist) library used for careful refactoring of critical code paths. It provides the `Experiment` struct used to define the conducted experiment and publishing utilities.

## About

The library aims to be as minimal as possible, pulling no external dependencies and using a bare minimum from the `std` library.

## Usage
`Experiment` struct represents a definition of the **control** and **experimental** execution block code paths.

The experiment is guided by the configuration specified during the `Experiment` construction and result observations are published internally using the `publish` function.

```rust
use scientisto::Experiment;
use tracing::info;

let expected: i32 = 1;
let result = Experiment::new("Test")
    .control(|| expected)
    .experiment(|| expected + 1)
    .publish(|o: &scientisto::Observation<i32, i32>| {
        tracing::info!("You can do any magic in the publisher")
     })
    .run();
```


## Limitations
- No defaults are provided for the `control` and `experiment` callbacks, they must be fully specified
- No `async` support
