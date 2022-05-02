use ::daggy::{Dag, NodeIndex};
use ::serde::{Deserialize, Serialize};
use daggy::petgraph::graph::DefaultIx;
use daggy::Walker;
use indexmap::IndexMap;
use linked_hash_map::LinkedHashMap;
use once_cell::sync::Lazy;
use std::collections::{HashMap, HashSet};
use tracing::Level;

use crate::attribute::OwnedMetadata;
use crate::layer::{ALL_LOGS, ALL_SPANS, SPAN_ID_TO_ROOT_AND_NODE_INDEX};
use crate::log::LogsRecorder;
use crate::record::{Record, RecordValue, RecordWithMetadata, Recorder};
use crate::LazyMutex;

pub(crate) static ALL_DAGS: LazyMutex<IndexMap<u64, Dag<u64, ()>>> = Lazy::new(Default::default);

#[derive(Debug)]
pub struct Filter {
    default_level: Level,
    targets: HashMap<String, Level>,
}

impl Filter {
    pub fn new(default_level: Level) -> Self {
        Self {
            default_level,
            targets: Default::default(),
        }
    }

    pub fn with_target(self, key: String, value: Level) -> Self {
        let mut targets = self.targets;
        targets.insert(key, value);
        Self { targets, ..self }
    }

    pub fn is_enabled(&self, metadata: &OwnedMetadata) -> bool {
        let mut for_target = self
            .targets
            .iter()
            .filter(|(key, _)| metadata.target.starts_with(key.as_str()))
            .collect::<Vec<_>>();
        for_target.sort_by(|(a, _), (b, _)| b.len().cmp(&a.len()));

        for_target
            .first()
            .map(|(_, level)| level)
            .unwrap_or(&&self.default_level)
            .ge(&&metadata
                .level
                .parse::<Level>()
                .expect("metadata level is invalid"))
    }
}

/// A tree which is effectively a Tree containing all the spans
///
/// It can't do much yet, except being Serialized, which comes in handy for snapshots.
#[derive(Debug, Serialize, PartialEq, Eq)]
pub struct Span {
    // the span id
    #[serde(skip_serializing)]
    id: u64,
    // the function name
    name: String,
    // the recorded variables and logs
    record: RecordWithMetadata,
    // the node's children
    children: LinkedHashMap<ChildKey, Span>,
}

#[derive(Default, Debug, Hash, PartialEq, Eq)]
struct ChildKey(String, usize);

impl Serialize for ChildKey {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        serializer.serialize_str(&self.0)
    }
}

impl Span {
    // Create a span from a name, a span_id, and recorded variables
    pub fn from(name: String, id: u64, record: RecordWithMetadata) -> Self {
        Self {
            name,
            id,
            record,
            children: Default::default(),
        }
    }
}
pub struct Report {
    root_index: NodeIndex,
    root_id: u64,
    dag: Dag<u64, (), DefaultIx>,
    spans: IndexMap<u64, Recorder>,
    logs: LogsRecorder,
    node_to_id: IndexMap<NodeIndex, u64>,
}

impl Report {
    pub fn from_root(root_node: u64) -> Self {
        let id_to_node = SPAN_ID_TO_ROOT_AND_NODE_INDEX.lock().unwrap().clone();
        let (global_root, root_node_index) = id_to_node
            .get(&root_node)
            .map(std::clone::Clone::clone)
            .expect("couldn't find rood node");

        let node_to_id: IndexMap<NodeIndex, u64> = id_to_node
            .into_iter()
            .filter_map(|(key, (root, value))| (root == global_root).then(|| (value, key)))
            .collect();

        let relevant_spans = node_to_id.values().cloned().collect::<HashSet<_>>();
        let spans = ALL_SPANS
            .lock()
            .unwrap()
            .clone()
            .into_iter()
            .filter(|(span_id, _)| relevant_spans.contains(span_id))
            .collect();
        let logs = ALL_LOGS.lock().unwrap().for_spans(relevant_spans);

        let dag = ALL_DAGS
            .lock()
            .unwrap()
            .get(&global_root)
            .expect("no dag for root")
            .clone();

        Self {
            root_index: root_node_index,
            root_id: root_node,
            dag,
            spans,
            node_to_id,
            logs,
        }
    }

    pub fn logs(&self, filter: &Filter) -> Records {
        if let Some(recorder) = self.spans.get(&self.root_id) {
            let mut contents = recorder.contents(filter);
            contents.append(
                self.logs
                    .record_for_span_id_and_filter(self.root_id, filter),
            );

            let mut records: Vec<_> = contents.entries().cloned().collect();

            self.dfs_logs_insert(&mut records, self.root_index, filter);

            Records::new(records)
        } else {
            Default::default()
        }
    }

    pub fn spans(&self, filter: &Filter) -> Span {
        if let Some(recorder) = self.spans.get(&self.root_id) {
            let metadata = recorder
                .metadata()
                .as_ref()
                .map(std::clone::Clone::clone)
                .expect("recorder without metadata");
            let span_name = format!("{}::{}", metadata.target, metadata.name);

            let mut root_span = Span::from(span_name, self.root_id, recorder.contents(filter));

            self.dfs_span_insert(&mut root_span, self.root_index, filter);

            root_span
        } else {
            Span::from("root".to_string(), 0, RecordWithMetadata::for_root())
        }
    }

    fn dfs_logs_insert(&self, records: &mut Vec<Record>, current_node: NodeIndex, filter: &Filter) {
        for child_node in self.sorted_children(current_node) {
            let child_id = self
                .node_to_id
                .get(&child_node)
                .expect("couldn't find span id for node");

            let mut child_record = self
                .spans
                .get(child_id)
                .expect("graph and hashmap are tied; qed")
                .contents(filter);

            child_record.append(self.logs.record_for_span_id_and_filter(*child_id, filter));
            records.extend(child_record.entries().cloned().into_iter());
            self.dfs_logs_insert(records, child_node, filter);
        }
    }

    fn dfs_span_insert(&self, current_span: &mut Span, current_node: NodeIndex, filter: &Filter) {
        current_span.children = self
            .sorted_children(current_node)
            .flat_map(|child_node| {
                let child_id = self
                    .node_to_id
                    .get(&child_node)
                    .expect("couldn't find span id for node");
                let child_recorder = self
                    .spans
                    .get(child_id)
                    .expect("graph and hashmap are tied; qed");

                let metadata = child_recorder
                    .metadata()
                    .expect("couldn't find metadata for child record");

                let span_name = format!("{}::{}", metadata.target, metadata.name);
                let mut contents = child_recorder.contents(filter);
                contents.append(self.logs.record_for_span_id_and_filter(*child_id, filter));

                if !filter.is_enabled(metadata) {
                    // We continue to fetch children spans with an enabled filter
                    let mut child_span = Span::from(span_name, *child_id, contents);
                    self.dfs_span_insert(&mut child_span, child_node, filter);

                    child_span
                        .children
                        .into_iter()
                        .collect::<Vec<(ChildKey, Span)>>()
                } else {
                    let mut child_span = Span::from(span_name.clone(), *child_id, contents);
                    self.dfs_span_insert(&mut child_span, child_node, filter);

                    vec![(ChildKey(span_name, child_node.index()), child_span)]
                }
            })
            .collect();
    }

    fn sorted_children(&self, node: NodeIndex) -> impl Iterator<Item = NodeIndex> {
        let mut children = self
            .dag
            .children(node)
            .iter(&self.dag)
            .map(|(_, node)| node)
            .collect::<Vec<_>>();
        children.sort();

        children.into_iter()
    }
}

/// A Vec of log entries.
#[derive(Debug, Serialize, Deserialize, Clone, Default, PartialEq, Eq)]
pub struct Records(Vec<Record>);

impl Records {
    /// Create a Records from log entries
    pub fn new(records: Vec<Record>) -> Self {
        Self(records)
    }

    /// check if log message has been stored with the given payload.
    pub fn contains_message(&self, lookup: impl AsRef<str>) -> bool {
        self.contains_value("message", RecordValue::Debug(lookup.as_ref().to_string()))
    }

    /// check if log entry (this can be span attributes or log messages) has been stored with the given payload.
    pub fn contains_value(&self, field_name: impl AsRef<str>, lookup: RecordValue) -> bool {
        self.0
            .iter()
            .any(|(field, value)| field.as_str() == field_name.as_ref() && value == &lookup)
    }
}
