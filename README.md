# test-span

A macro and utilities to do snapshot tests on tracing spans.

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
    // you can specify the span / log level you would like to track like this:
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
            tracing::info!("will only show up in get_all_logs!");
        })
        .join()
        .unwrap();

        // get_all_logs takes a filter Level
        let all_logs = test_span::get_all_logs(&tracing::Level::INFO);
        assert!(all_logs.contains_message("will only show up in get_all_logs!"));
    }
```

## License

<sup>
Licensed under either of <a href="LICENSE-APACHE">Apache License, Version
2.0</a> or <a href="LICENSE-MIT">MIT license</a> at your option.
</sup>

<br>

<sub>
Unless you explicitly state otherwise, any contribution intentionally submitted
for inclusion in this crate by you, as defined in the Apache-2.0 license, shall
be dual licensed as above, without any additional terms or conditions.
</sub>
