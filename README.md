# test-span

A macro and utilities to do snapshot tests on tracing spans.

<p>
  <a href="https://docs.rs/test-span">
      <img src="https://docs.rs/test-span/badge.svg" alt="docs">
  </a>
  <a href="https://app.circleci.com/pipelines/github/apollographql/test-span">
      <img src="https://circleci.com/gh/circleci/circleci-docs.svg?style=shield" alt="Build status">
  </a>
  <a href="LICENSE-APACHE">
    <img
    src="https://img.shields.io/badge/license-apache2-green.svg" alt="Apache 2.0 License">
  </a>
  <a href="LICENSE-MIT">
    <img
    src="https://img.shields.io/badge/license-mit-blue.svg" alt="MIT License">
  </a>
</p>

## How to use

Refer to the [tests](test-span/tests/tests.rs) for a more exhaustive list of features and behaviors:

```rust
use test_span::prelude::*;

#[test_span]
fn a_test() {
    do_something();

    // test_span provides your with three functions:
    let spans = get_spans();
    let logs = get_logs();
    // you can get both in one call
    let (spans, logs) = get_telemetry();

    // This plays well with insta snapshots:
    insta::assert_json_snapshot!(logs);
    insta::assert_json_snapshot!(spans);
}

// Test span plays well with async
#[test_span(tokio::test)]
// you can specify the span / log level
// you would like to track like this:
#[level(tracing::Level::INFO)]
async fn an_sync_test() {
    do_something_async().await;
    // You still get access to each function
    let spans = get_spans();
    let logs = get_logs();
    let (spans, logs) = get_telemetry();
}
```

## Limitations

Spans and logs are hard to track across thread spawns. However we're providing you with a log dump you can check:

```rust
#[test_span]
fn track_across_threads() {
    std::thread::spawn(|| {
        tracing::info!("only in get_all_logs!");
    })
    .join()
    .unwrap();

    let logs = get_logs();
    // not in get_logs()
    assert!(!logs.contains_message("only in get_all_logs!");

    // get_all_logs takes a filter Level
    let all_logs = test_span::get_all_logs(&tracing::Level::INFO);
    assert!(all_logs.contains_message("only in get_all_logs!"));
}
```

## Contributing

More information can be found in [the contribution docs](CONTRIBUTING.md)

## License

<sup>
Licensed under either of [Apache License, Version
2.0](LICENSE-APACHE) or [MIT license](LICENSE-MIT) at your option.
</sup>

<br>

<sub>
Unless you explicitly state otherwise, any contribution intentionally submitted
for inclusion in this crate by you, as defined in the Apache-2.0 license, shall
be dual licensed as above, without any additional terms or conditions.
</sub>
