//! Replay `tracing` recordings.
//!
//! # Overview
//!
//! The `tracing-replay` crate is the matching pair of the `tracing-rec` crate.
//!
//! `tracing-rec` provides a [`tracing-subscriber`] layer which can record event and span traces
//! into a serialized format.
//!
//! `tracing-replay` can then take the serialized format and replay it into the current
//! [`tracing`] dispatcher.
//!
//! # Usage
//!
//! The recorded traces from a file at a provided path will be replayed into the current
//! [`Dispatch`].
//!
//! ```
//! # let temp_dir = tempfile::tempdir().unwrap();
//! # let path_buf = temp_dir.path().join("recording.tracing");
//! # let recording_path = path_buf.to_str().unwrap();
//! # {
//! #    use std::io::Write;
//! #    let mut file = std::fs::File::create(recording_path).unwrap();
//! #    writeln!(file, "{}", r#"{"meta":{"timestamp_s":1708644606,"timestamp_subsec_us":74773,"thread_id":"ThreadId(1)","thread_name":"main"},"trace":{"RegisterCallsite":{"id":4435670072,"name":"event tracing-rec/examples/events.rs:8","target":"events","level":"Info","module_path":"events","file":"tracing-rec/examples/events.rs","line":8,"fields":["message"],"kind":"Event"}}}"#);
//! #    writeln!(file, "{}", r#"{"meta":{"timestamp_s":1708644606,"timestamp_subsec_us":74908,"thread_id":"ThreadId(1)","thread_name":"main"},"trace":{"Event":{"fields":[["message","I am an info event!"]],"metadata":{"id":4435670072,"name":"event tracing-rec/examples/events.rs:8","target":"events","level":"Info","module_path":"events","file":"tracing-rec/examples/events.rs","line":8,"fields":["message"],"kind":"Event"},"parent":"Current"}}}"#);
//! # }
//!
//! let mut replay = tracing_replay::Replay::new();
//! let result = replay.replay_file(recording_path);
//!
//! println!("{:?}", result);
//! assert!(result.is_ok());
//! # temp_dir.close().unwrap();
//! ```
//!
//! # Supported Rust Versions
//!
//! `tracing-replay` is built against the latest stable release. The minimum supported version is
//! 1.76. The current version of `tracing-replay` is not guaranteed to build on Rust versions
//! earlier than the minimum supported version.
//!
//! # License
//!
//! This project is licensed under the [MIT license].
//!
//! [MIT license]: https://github.com/hds/tracing-rec-replay/blob/main/LICENSE
//!
//! # Contribution
//!
//! Unless you explicitly state otherwise, any contribution intentionally submitted for inclusion
//! in `tracing-replay` by you, shall be licensed as MIT, without any additional terms or
//! conditions.
//!
//! [`Dispatch`]: struct@tracing::Dispatch
//! [`tracing-subscriber`]: https://docs.rs/tracing-subscriber/latest/tracing_subscriber/
#![allow(clippy::many_single_char_names)]

use std::{
    collections::HashMap,
    error, fmt,
    fs::File,
    io::{self, BufReader},
    sync::{Arc, Mutex},
};

use tracing::span::Attributes;
use tracing_core::{field, span, Event, Metadata};

mod callsite;
mod recording;

use callsite::Cs;
use recording::{Trace, TraceRecord};

/// Replay coordinator.
///
/// An instantiation of this object can replay a tracing recording. See [`replay_file`] for details
/// and examples.
///
/// [`replay_file`]: fn@Self::replay_file
#[derive(Debug)]
pub struct Replay {
    store: Arc<Mutex<HashMap<u64, &'static Metadata<'static>>>>,
    callsites: Arc<Mutex<HashMap<recording::SpanId, u64>>>,
    span_ids: Arc<Mutex<HashMap<recording::SpanId, span::Id>>>,
}

impl Replay {
    #[must_use = "A replayer doesn't do anything until it is given a recording to replay"]
    pub fn new() -> Self {
        Self {
            store: Arc::new(Mutex::new(HashMap::new())),
            callsites: Arc::new(Mutex::new(HashMap::new())),
            span_ids: Arc::new(Mutex::new(HashMap::new())),
        }
    }

    /// Replays a tracing recording file through the default dispatcher.
    ///
    /// The file at `path` is read and the trace records stored in the file are replayed one by
    /// one.
    ///
    /// # Errors
    ///
    /// This method will return an error if the file at the provided path cannot be read or if
    /// individual records cannot be read or deserialized.
    ///
    /// # Examples
    ///
    /// ```
    /// # let temp_dir = tempfile::tempdir().unwrap();
    /// # let path_buf = temp_dir.path().join("recording.tracing");
    /// # let recording_path = path_buf.to_str().unwrap();
    /// # {
    /// #    use std::io::Write;
    /// #    let mut file = std::fs::File::create(recording_path).unwrap();
    /// #    writeln!(file, "{}", r#"{"meta":{"timestamp_s":1708644606,"timestamp_subsec_us":74773,"thread_id":"ThreadId(1)","thread_name":"main"},"trace":{"RegisterCallsite":{"id":4435670072,"name":"event tracing-rec/examples/events.rs:8","target":"events","level":"Info","module_path":"events","file":"tracing-rec/examples/events.rs","line":8,"fields":["message"],"kind":"Event"}}}"#);
    /// #    writeln!(file, "{}", r#"{"meta":{"timestamp_s":1708644606,"timestamp_subsec_us":74908,"thread_id":"ThreadId(1)","thread_name":"main"},"trace":{"Event":{"fields":[["message","I am an info event!"]],"metadata":{"id":4435670072,"name":"event tracing-rec/examples/events.rs:8","target":"events","level":"Info","module_path":"events","file":"tracing-rec/examples/events.rs","line":8,"fields":["message"],"kind":"Event"},"parent":"Current"}}}"#);
    /// # }
    ///
    /// let mut replay = tracing_replay::Replay::new();
    /// let result = replay.replay_file(recording_path);
    ///
    /// println!("{:?}", result);
    /// assert!(result.is_ok());
    /// # temp_dir.close().unwrap();
    /// ```
    pub fn replay_file(&mut self, path: &str) -> Result<ReplaySummary, ReplayFileError> {
        use std::io::prelude::*;

        let file =
            File::open(path).map_err(|io_err| ReplayFileError::CannotOpenFile { inner: io_err })?;
        let reader = BufReader::new(file);

        let mut record_count = 0;
        for (line_index, line) in reader.lines().enumerate() {
            let line = &line.map_err(|io_err| ReplayFileError::CannotReadLine {
                inner: io_err,
                line_index,
            })?;
            let trace_record = serde_json::from_str(line).map_err(|err| {
                ReplayFileError::CannotDeserializeRecord {
                    inner: err,
                    line_index,
                    line: line.clone(),
                }
            })?;

            self.dispatch_trace(trace_record);
            record_count += 1;
        }

        Ok(ReplaySummary { record_count })
    }
}

#[non_exhaustive]
#[derive(Debug)]
pub struct ReplaySummary {
    pub record_count: usize,
}

#[non_exhaustive]
#[derive(Debug)]
pub enum ReplayFileError {
    CannotOpenFile {
        inner: io::Error,
    },
    CannotReadLine {
        inner: io::Error,
        line_index: usize,
    },
    CannotDeserializeRecord {
        inner: serde_json::Error,
        line_index: usize,
        line: String,
    },
}

impl fmt::Display for ReplayFileError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{self:?}")
    }
}

impl error::Error for ReplayFileError {}

impl Replay {
    fn get_or_create_metadata(
        &self,
        rec_metadata: recording::Metadata,
    ) -> &'static Metadata<'static> {
        let mut guard = self
            .store
            .lock()
            .expect("replay internal state (store) has become corrupted.");

        let metadata: &'static Metadata = (*guard)
            .entry(rec_metadata.id)
            .or_insert_with(|| Box::leak(Box::new(rec_metadata.into())));

        metadata
    }

    fn set_span_id_callsite(&self, rec_span_id: recording::SpanId, callsite_id: u64) {
        let mut guard = self
            .callsites
            .lock()
            .expect("replay internal state (callsites) has become corrupted.");

        (*guard).insert(rec_span_id, callsite_id);
    }

    fn get_metadata_by_span_id(
        &self,
        rec_span_id: recording::SpanId,
    ) -> Option<&'static Metadata<'static>> {
        let callsite_id = {
            let guard = self
                .callsites
                .lock()
                .expect("replay internal state (callsites) has become corrupted.");

            (*guard).get(&rec_span_id).copied()
        }?;

        let guard = self
            .store
            .lock()
            .expect("replay internal state (store) has become corrupted.");

        (*guard).get(&callsite_id).copied()
    }

    fn dispatch_trace(&self, record: TraceRecord) {
        match record.trace {
            Trace::RegisterCallsite(rec_metadata) => self.register_callsite(rec_metadata),
            Trace::Event(rec_event) => self.event(rec_event),
            Trace::NewSpan(rec_new_span) => self.new_span(rec_new_span),
            Trace::Enter(rec_span_id) => self.enter_span(rec_span_id),
            Trace::Exit(rec_span_id) => self.exit_span(rec_span_id),
            Trace::Close(rec_span_id) => {
                self.try_close_span(rec_span_id);
            }
            Trace::FollowsFrom(rec_follows_from) => self.follows_from(&rec_follows_from),
            Trace::Record(rec_record_values) => self.record_values(&rec_record_values),
        }
    }

    fn register_callsite(&self, rec_metadata: recording::Metadata) {
        let metadata = self.get_or_create_metadata(rec_metadata);
        tracing::dispatcher::get_default(move |dispatch| dispatch.register_callsite(metadata));
    }

    fn new_span(&self, rec_new_span: recording::NewSpan) {
        let callsite_id = rec_new_span.metadata.id;
        let metadata = self.get_or_create_metadata(rec_new_span.metadata);
        self.set_span_id_callsite(rec_new_span.id, callsite_id);

        tracing::dispatcher::get_default(move |dispatch| {
            if !dispatch.enabled(metadata) {
                return;
            }

            let fields = metadata.fields();
            let mut values = Vec::new();
            let mut field_vec = Vec::new();
            for (field_name, value) in &rec_new_span.fields {
                field_vec.push((fields.field(field_name), value));
            }

            for (field, value) in &field_vec {
                if let Some(field) = field {
                    values.push((field, Some(value as &dyn field::Value)));
                }
            }

            let parent = &rec_new_span.parent;
            let span_id = match *values.as_slice() {
                [] => dispatch_new_span(dispatch, metadata, parent, []),
                [a] => dispatch_new_span(dispatch, metadata, parent, [a]),
                [a, b] => dispatch_new_span(dispatch, metadata, parent, [a, b]),
                [a, b, c] => dispatch_new_span(dispatch, metadata, parent, [a, b, c]),
                [a, b, c, d] => dispatch_new_span(dispatch, metadata, parent, [a, b, c, d]),
                [a, b, c, d, e] => dispatch_new_span(dispatch, metadata, parent, [a, b, c, d, e]),
                [a, b, c, d, e, f] => {
                    dispatch_new_span(dispatch, metadata, parent, [a, b, c, d, e, f])
                }
                [a, b, c, d, e, f, g] => {
                    dispatch_new_span(dispatch, metadata, parent, [a, b, c, d, e, f, g])
                }
                [a, b, c, d, e, f, g, h] => {
                    dispatch_new_span(dispatch, metadata, parent, [a, b, c, d, e, f, g, h])
                }
                [a, b, c, d, e, f, g, h, i, ..] => {
                    dispatch_new_span(dispatch, metadata, parent, [a, b, c, d, e, f, g, h, i])
                }
            };

            // Store a mapping from the recorded span::Id to the one that `tracing` has given us
            // during this replay. We will need to look up this mapping to replay traces that
            // reference this new span by Id (enter, exit, ...).
            {
                let mut guard = self
                    .span_ids
                    .lock()
                    .expect("replay internal state has become corrupted.");
                debug_assert!(
                    (*guard).get(&rec_new_span.id).is_none(),
                    "new span recorded span::Id that has already been seen!"
                );
                (*guard).insert(rec_new_span.id, span_id);
            }
        });
    }

    fn enter_span(&self, rec_span_id: recording::SpanId) {
        let span_id = self
            .get_replay_span_id(rec_span_id)
            .expect("no replay span::Id found, is the recording complete?");
        tracing::dispatcher::get_default(|dispatch| dispatch.enter(&span_id));
    }

    fn exit_span(&self, rec_span_id: recording::SpanId) {
        let span_id = self
            .get_replay_span_id(rec_span_id)
            .expect("no replay span::Id found, is the recording complete?");

        tracing::dispatcher::get_default(|dispatch| dispatch.exit(&span_id));
    }

    fn try_close_span(&self, rec_span_id: recording::SpanId) -> bool {
        let span_id = self
            .get_replay_span_id(rec_span_id)
            .expect("no replay span::Id found, is the recording complete?");
        tracing::dispatcher::get_default(move |dispatch| dispatch.try_close(span_id.clone()))
    }

    fn follows_from(&self, rec_follows_from: &recording::FollowsFrom) {
        let Some(cause_span_id) = self.get_replay_span_id(rec_follows_from.cause_id) else {
            return;
        };
        let Some(effect_span_id) = self.get_replay_span_id(rec_follows_from.effect_id) else {
            return;
        };
        tracing::dispatcher::get_default(move |dispatch| {
            dispatch.record_follows_from(&effect_span_id, &cause_span_id);
        });
    }

    fn record_values(&self, rec_record_values: &recording::RecordValues) {
        let Some(span_id) = self.get_replay_span_id(rec_record_values.id) else {
            return;
        };
        let Some(metadata) = self.get_metadata_by_span_id(rec_record_values.id) else {
            return;
        };

        let fields = metadata.fields();
        let mut values = Vec::new();
        let mut field_vec = Vec::new();
        for (field_name, value) in &rec_record_values.fields {
            field_vec.push((fields.field(field_name), value));
        }

        for (field, value) in &field_vec {
            if let Some(field) = field {
                values.push((field, Some(value as &dyn field::Value)));
            }
        }

        tracing::dispatcher::get_default(move |dispatch| match *values.as_slice() {
            [] => dispatch_record(dispatch, metadata, &span_id, []),
            [a] => dispatch_record(dispatch, metadata, &span_id, [a]),
            [a, b] => dispatch_record(dispatch, metadata, &span_id, [a, b]),
            [a, b, c] => dispatch_record(dispatch, metadata, &span_id, [a, b, c]),
            [a, b, c, d] => dispatch_record(dispatch, metadata, &span_id, [a, b, c, d]),
            [a, b, c, d, e] => dispatch_record(dispatch, metadata, &span_id, [a, b, c, d, e]),
            [a, b, c, d, e, f] => dispatch_record(dispatch, metadata, &span_id, [a, b, c, d, e, f]),
            [a, b, c, d, e, f, g] => {
                dispatch_record(dispatch, metadata, &span_id, [a, b, c, d, e, f, g]);
            }
            [a, b, c, d, e, f, g, h] => {
                dispatch_record(dispatch, metadata, &span_id, [a, b, c, d, e, f, g, h]);
            }
            [a, b, c, d, e, f, g, h, i, ..] => {
                dispatch_record(dispatch, metadata, &span_id, [a, b, c, d, e, f, g, h, i]);
            }
        });
    }

    fn event(&self, rec_event: recording::Event) {
        let metadata = self.get_or_create_metadata(rec_event.metadata);
        tracing::dispatcher::get_default(move |dispatch| {
            let enabled = dispatch.enabled(metadata);
            if enabled {
                let fields = metadata.fields();
                let mut values = Vec::new();
                let mut field_vec = Vec::new();
                for (field_name, value) in &rec_event.fields {
                    field_vec.push((fields.field(field_name), value));
                }

                for (field, value) in &field_vec {
                    if let Some(field) = field {
                        values.push((field, Some(value as &dyn field::Value)));
                    }
                }

                let parent = &rec_event.parent;
                match *values.as_slice() {
                    [] => dispatch_event(dispatch, metadata, parent, []),
                    [a] => dispatch_event(dispatch, metadata, parent, [a]),
                    [a, b] => dispatch_event(dispatch, metadata, parent, [a, b]),
                    [a, b, c] => dispatch_event(dispatch, metadata, parent, [a, b, c]),
                    [a, b, c, d] => dispatch_event(dispatch, metadata, parent, [a, b, c, d]),
                    [a, b, c, d, e] => dispatch_event(dispatch, metadata, parent, [a, b, c, d, e]),
                    [a, b, c, d, e, f] => {
                        dispatch_event(dispatch, metadata, parent, [a, b, c, d, e, f]);
                    }
                    [a, b, c, d, e, f, g] => {
                        dispatch_event(dispatch, metadata, parent, [a, b, c, d, e, f, g]);
                    }
                    [a, b, c, d, e, f, g, h] => {
                        dispatch_event(dispatch, metadata, parent, [a, b, c, d, e, f, g, h]);
                    }
                    [a, b, c, d, e, f, g, h, i, ..] => {
                        dispatch_event(dispatch, metadata, parent, [a, b, c, d, e, f, g, h, i]);
                    }
                }
            }
        });
    }

    fn get_replay_span_id(&self, rec_span_id: recording::SpanId) -> Option<span::Id> {
        let guard = self
            .span_ids
            .lock()
            .expect("replay internal state has become corrupted.");
        (*guard).get(&rec_span_id).cloned()
    }
}

fn dispatch_event<const N: usize>(
    dispatch: &tracing::Dispatch,
    metadata: &'static Metadata<'static>,
    parent: &recording::Parent,
    value: [(&field::Field, Option<&dyn tracing::Value>); N],
) {
    let value_set = metadata.fields().value_set(&value);
    let event = match parent {
        recording::Parent::Current => Event::new(metadata, &value_set),
        recording::Parent::Root => Event::new_child_of(None, metadata, &value_set),
        recording::Parent::Explicit(parent_id) => {
            Event::new_child_of(Some(span::Id::from_u64(*parent_id)), metadata, &value_set)
        }
    };
    dispatch.event(&event);
}

fn dispatch_new_span<const N: usize>(
    dispatch: &tracing::Dispatch,
    metadata: &'static Metadata<'static>,
    parent: &recording::Parent,
    value: [(&field::Field, Option<&dyn tracing::Value>); N],
) -> span::Id {
    let value_set = metadata.fields().value_set(&value);
    let attr = match parent {
        recording::Parent::Current => Attributes::new(metadata, &value_set),
        recording::Parent::Root => Attributes::new_root(metadata, &value_set),
        &recording::Parent::Explicit(parent_id) => {
            Attributes::child_of(span::Id::from_u64(parent_id), metadata, &value_set)
        }
    };
    dispatch.new_span(&attr)
}

fn dispatch_record<const N: usize>(
    dispatch: &tracing::Dispatch,
    metadata: &'static Metadata<'static>,
    span_id: &span::Id,
    value: [(&field::Field, Option<&dyn tracing::Value>); N],
) {
    let value_set = metadata.fields().value_set(&value);
    let record = span::Record::new(&value_set);
    dispatch.record(span_id, &record);
}

impl Default for Replay {
    fn default() -> Self {
        Self::new()
    }
}

impl From<recording::Metadata> for Metadata<'static> {
    fn from(val: recording::Metadata) -> Self {
        let cs: &'static Cs = Box::leak(Box::new(Cs::new(val.id)));

        // self.fields
        let fields: Vec<&'static str> = val
            .fields
            .into_iter()
            .map(|f| Box::leak(Box::new(f)) as &'static str)
            .collect();

        tracing::Metadata::new(
            leak(val.name),
            leak(val.target),
            val.level.into(),
            val.file.map(|s| leak(s) as &'static str),
            val.line,
            val.module_path.map(|s| leak(s) as &'static str),
            tracing::field::FieldSet::new(leak(fields), tracing_core::identify_callsite!(cs)),
            val.kind.into(),
        )
    }
}

fn leak<T>(obj: T) -> &'static T {
    Box::leak(Box::new(obj))
}

// fn new_span(
//     metadata_id: u64,
//     span_name: &'static str,
//     store: Arc<Mutex<HashMap<u64, Metadata<'static>>>>,
// ) -> span::Id {
//     tracing::dispatcher::get_default(move |dispatch| {
//         let metadata = match metadata_or_create(metadata_id, span_name, &store.clone()) {
//             MetadataEntry::New(metadata) => {
//                 dispatch.register_callsite(metadata);
//                 metadata
//             }
//             MetadataEntry::Existing(metadata) => metadata,
//         };

//         let fields = metadata.fields();
//         let field = fields.field("field").unwrap();
//         let values = [(&field, Some(&"field-value" as &dyn field::Value))];
//         let value_set = metadata.fields().value_set(&values);

//         let span_id = tracing::dispatcher::get_default(move |dispatch| {
//             let span_attributes = Attributes::new(metadata, &value_set);
//             dispatch.new_span(&span_attributes)
//         });

//         // let span = Span::new(metadata, &value_set);
//         // span.id().unwrap()

//         span_id
//     })
// }
