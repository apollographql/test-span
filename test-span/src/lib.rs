//! the test span library provides you with two functions:
//!
//! `get_logs()` that returns [`prelude::Records`]
//!
//! `get_span()` that returns a [`prelude::Span`],
//! Which can be serialized and used with [insta](https://crates.io/crates/insta) for snapshot tests.
//!  Refer to the tests.rs file to see how it behaves.
//!
//! Example:
//! ```ignore
//! #[test_span]
//! async fn test_it_works() {
//!   futures::join!(do_stuff(), do_stuff())
//! }
//!
//! #[tracing::instrument(name = "do_stuff", level = "info")]
//! async fn do_stuff() -> u8 {
//!     // ...
//!     do_stuff2().await;
//! }
//!
//! #[tracing::instrument(
//!     name = "do_stuff2",
//!     target = "my_crate::an_other_target",
//!     level = "info"
//! )]
//! async fn do_stuff_2(number: u8) -> u8 {
//!     // ...
//! }
//! ```
//! ```text
//! `get_span()` will provide you with:
//!
//!             ┌──────┐
//!             │ root │
//!             └──┬───┘
//!                │
//!        ┌───────┴───────┐
//!        ▼               ▼
//!   ┌──────────┐   ┌──────────┐
//!   │ do_stuff │   │ do_stuff │
//!   └────┬─────┘   └─────┬────┘
//!        │               │
//!        │               │
//!        ▼               ▼
//!  ┌───────────┐   ┌───────────┐
//!  │ do_stuff2 │   │ do_stuff2 │
//!  └───────────┘   └───────────┘
//! ```

use once_cell::sync::Lazy;
use std::sync::{Arc, Mutex};
use tracing::Id;
use tracing_core::dispatcher;
use tracing_subscriber::{prelude::__tracing_subscriber_SubscriberExt, util::SubscriberInitExt};
type LazyMutex<T> = Lazy<Arc<Mutex<T>>>;

mod attribute;
mod layer;
mod log;
mod record;
mod report;

pub use layer::Layer;
pub use record::{Record, RecordValue};
pub use report::{Filter, Records, Report, Span};

static INIT: Lazy<()> = Lazy::new(|| {
    if dispatcher::has_been_set() {
        dispatcher::get_default(|dispatcher| {
            assert!(dispatcher.is::<Layer>(), "A tracing global subscriber has already been set by an other crate than test-span, cannot proceed");
        })
    } else {
        let dispatcher = tracing_subscriber::registry().with(Layer {});
        dispatcher
            .try_init()
            .expect("couldn't set test-span subscriber as a default")
    }
});

/// `init_default` is the default way to call `with_targets`,
/// it sets up `Level::INFO` and looks for environment variables to filter spans.
pub fn init() {
    Lazy::force(&INIT);
}

/// Unlike its `get_logs` counterpart provided by the trace_span macro,
/// `get_all_logs` will return all of the module's tests logs.
pub fn get_all_logs(filter: &Filter) -> Records {
    let logs = layer::ALL_LOGS.lock().unwrap().clone();

    Records::new(logs.all_records_for_filter(filter))
}

/// Returns both the output of `get_spans_for_root` and `get_logs_for_root`
pub fn get_telemetry_for_root(root_id: &Id, filter: &Filter) -> (Span, Records) {
    let report = Report::from_root(root_id.into_u64());

    (report.spans(filter), report.logs(filter))
}

/// Returns a `Span`, a Tree containing all the spans that are children of `root_id`.
///
/// This function filters the `Span` children and `Records`,
/// to only return the ones that match the set verbosity level
pub fn get_spans_for_root(root_id: &Id, filter: &Filter) -> Span {
    Report::from_root(root_id.into_u64()).spans(filter)
}

/// Returns Records, which is a Vec, containing all entries recorded by children of `root_id`.
///
/// This function filters the `Records`to only return the ones that match the set verbosity level.
///
/// / ! \ Logs recorded in spawned threads won't appear here / ! \ use `get_all_logs` instead.
pub fn get_logs_for_root(root_id: &Id, filter: &Filter) -> Records {
    Report::from_root(root_id.into_u64()).logs(filter)
}

pub mod prelude {
    pub use crate::{get_all_logs, get_logs_for_root, get_spans_for_root, get_telemetry_for_root};
    pub use test_span_macro::test_span;
}

pub mod reexports {
    pub use daggy;
    pub use serde;
    pub use tracing;
    pub use tracing_futures;
    pub use tracing_subscriber;
}

#[cfg(test)]
mod test_span_doesnt_panic_tests {
    use super::*;

    #[test]
    fn init_with_already_set_test_span_global_subscriber_doesnt_panic() {
        tracing_subscriber::registry()
            .with(Layer {})
            .try_init()
            .unwrap();
        init();
    }
}
