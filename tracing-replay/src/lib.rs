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
    any::Any,
    collections::HashMap,
    error, fmt,
    fs::File,
    io::{self, BufReader},
    sync::{mpsc, Arc, Mutex},
    thread::{self, JoinHandle},
    time::{Duration, SystemTime, UNIX_EPOCH},
};

use proxy::{EventProxy, RecordProxy};
use tracing_core::{field, span, Metadata};

mod callsite;
mod proxy;
mod recording;

use crate::{
    callsite::Cs,
    proxy::{DispatchProxy, NewSpanProxy},
    recording::{Trace, TraceRecord},
};

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
    span_ids: Arc<Mutex<HashMap<recording::SpanId, MappedSpanId>>>,
    threads: HashMap<String, ThreadDispatcherHandle>,
    replay_time_delta: Duration,
}

#[derive(Debug)]
enum MappedSpanId {
    Pending,
    Mapped(span::Id),
}

impl Replay {
    #[must_use = "A replayer doesn't do anything until it is given a recording to replay"]
    pub fn new() -> Self {
        Self {
            store: Arc::new(Mutex::new(HashMap::new())),
            callsites: Arc::new(Mutex::new(HashMap::new())),
            span_ids: Arc::new(Mutex::new(HashMap::new())),
            threads: HashMap::new(),
            replay_time_delta: Duration::from_nanos(0),
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
            let trace_record: TraceRecord = serde_json::from_str(line).map_err(|err| {
                ReplayFileError::CannotDeserializeRecord {
                    inner: err,
                    line_index,
                    line: line.clone(),
                }
            })?;

            if line_index == 0 {
                let now_since_epoch = SystemTime::now().duration_since(UNIX_EPOCH).unwrap();
                let recording_since_epoch = Duration::new(
                    trace_record.meta.timestamp_s,
                    trace_record.meta.timestamp_subsec_us,
                );

                // Set the delta between now and the recording time. We'll use this to delay
                // replays and make them run on the same schedule as the recording.
                self.replay_time_delta = now_since_epoch.saturating_sub(recording_since_epoch);
            }

            self.dispatch_trace(trace_record);
            record_count += 1;
        }

        Ok(ReplaySummary { record_count })
    }

    /// Close the replay and check for errors.
    ///
    /// Since much of the work of replaying a [`tracing`] recording happens on other threads, work
    /// may be ongoing when [`replay_file`] returns. This is desireable in the case that another
    /// file constituting more traces from the same recording is to be replayed directly
    /// afterwards.
    ///
    /// Calling this method waits for the dispatcher threads to complete and then tears them down.
    ///
    /// # Errors
    ///
    /// If any of the dispatcher threads panicked, the resulting messages are returned in
    /// `ReplayCloseError`. All errors are collected into a vec together with their
    /// recording thread Id.
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
    /// let _replay_result = replay.replay_file(recording_path);
    ///
    /// let close_result = replay.close();
    /// assert!(close_result.is_ok());
    /// # temp_dir.close().unwrap();
    /// ```
    pub fn close(&mut self) -> Result<(), ReplayCloseError> {
        let mut errors = Vec::new();
        for (key, handle) in self.threads.drain() {
            match handle.trace_tx.send(DispatchableContainer::End) {
                Ok(()) => match handle.join_handle.join() {
                    Ok(()) => {}
                    Err(join_error) => errors.push((key, join_error)),
                },
                Err(send_error) => {
                    errors.push((key, Box::new(send_error)));
                }
            }
        }

        if errors.is_empty() {
            Ok(())
        } else {
            Err(ReplayCloseError { threads: errors })
        }
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

#[derive(Debug)]
pub struct ReplayCloseError {
    threads: Vec<(String, Box<dyn Any + Send + 'static>)>,
}

impl fmt::Display for ReplayCloseError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "ReplayCloseError:")?;
        for (thread_id, error) in &self.threads {
            write!(f, " - {thread_id}: {error:?}")?;
        }

        Ok(())
    }
}

impl error::Error for ReplayCloseError {}

impl Default for Replay {
    fn default() -> Self {
        Self::new()
    }
}

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

    fn dispatch_trace(&mut self, record: TraceRecord) {
        let trace_tx = {
            let handle = self
                .threads
                .entry(record.meta.thread_id)
                .or_insert_with_key(|thread_id| {
                    let (tx, rx) = mpsc::channel();
                    let thread_dispatcher = ThreadDispatcher {
                        rec_id: thread_id.clone(),
                        trace_rx: rx,
                        span_ids: Arc::clone(&self.span_ids),
                    };
                    let join_handle = thread::Builder::new()
                        .name(record.meta.thread_name.unwrap_or_default())
                        .spawn(move || {
                            thread_dispatcher.run();
                        })
                        .unwrap_or_else(|err| {
                            panic!(
                                "failed to create replay thread '{thread_id}'. \
                                Cannot faithfully reproduce traces. Error: {err}"
                            );
                        });
                    ThreadDispatcherHandle {
                        trace_tx: tx,
                        join_handle,
                    }
                });
            handle.trace_tx.clone()
        };

        let record_since_epoch =
            Duration::new(record.meta.timestamp_s, record.meta.timestamp_subsec_us);
        let replay_since_epoch = record_since_epoch
            .checked_add(self.replay_time_delta)
            .unwrap_or_else(|| SystemTime::now().duration_since(UNIX_EPOCH).unwrap());

        let container = match record.trace {
            Trace::RegisterCallsite(rec_metadata) => {
                let metadata = self.get_or_create_metadata(rec_metadata);
                DispatchableContainer::Trace {
                    timestamp: replay_since_epoch,
                    trace: DispatchableTrace::RegisterCallsite(DispatchableMetadata(metadata)),
                }
            }
            Trace::Event(rec_event) => {
                let dis_event = self.event(rec_event);
                DispatchableContainer::Trace {
                    timestamp: replay_since_epoch,
                    trace: DispatchableTrace::Event(dis_event),
                }
            }
            Trace::NewSpan(rec_new_span) => {
                let dis_new_span = self.new_span(rec_new_span);
                DispatchableContainer::Trace {
                    timestamp: replay_since_epoch,
                    trace: DispatchableTrace::NewSpan(dis_new_span),
                }
            }
            Trace::Enter(rec_span_id) => DispatchableContainer::Trace {
                timestamp: replay_since_epoch,
                trace: DispatchableTrace::Enter(DispatchableSpanId(rec_span_id)),
            },
            Trace::Exit(rec_span_id) => DispatchableContainer::Trace {
                timestamp: replay_since_epoch,
                trace: DispatchableTrace::Exit(DispatchableSpanId(rec_span_id)),
            },
            Trace::Close(rec_span_id) => DispatchableContainer::Trace {
                timestamp: replay_since_epoch,
                trace: DispatchableTrace::Close(DispatchableSpanId(rec_span_id)),
            },
            Trace::Record(rec_record_values) => {
                let Some(metadata) = self.get_metadata_by_span_id(rec_record_values.id) else {
                    return;
                };
                DispatchableContainer::Trace {
                    timestamp: replay_since_epoch,
                    trace: DispatchableTrace::Record(DispatchableRecordValues {
                        id: rec_record_values.id,
                        metadata,
                        fields: rec_record_values.fields,
                    }),
                }
            }
            Trace::FollowsFrom(rec_follows_from) => DispatchableContainer::Trace {
                timestamp: replay_since_epoch,
                trace: DispatchableTrace::FollowsFrom(DispatchableFollowsFrom {
                    cause_id: rec_follows_from.cause_id,
                    effect_id: rec_follows_from.effect_id,
                }),
            },
        };
        if let Err(err) = trace_tx.send(container) {
            println!("failed to send container: {err}");
        };
    }

    fn new_span(&self, rec_new_span: recording::NewSpan) -> DispatchableNewSpan {
        let callsite_id = rec_new_span.metadata.id;
        let metadata = self.get_or_create_metadata(rec_new_span.metadata);
        self.set_span_id_callsite(rec_new_span.id, callsite_id);

        {
            let mut guard = self
                .span_ids
                .lock()
                .expect("replay internal state has become corrupted.");
            debug_assert!(
                (*guard).get(&rec_new_span.id).is_none(),
                "new span recorded span::Id that has already been seen!"
            );
            (*guard).insert(rec_new_span.id, MappedSpanId::Pending);
        }

        DispatchableNewSpan {
            id: rec_new_span.id,
            metadata,
            fields: rec_new_span.fields,
            parent: rec_new_span.parent,
        }
    }

    fn event(&self, rec_event: recording::Event) -> DispatchableEvent {
        let metadata = self.get_or_create_metadata(rec_event.metadata);
        DispatchableEvent {
            metadata,
            fields: rec_event.fields,
            parent: rec_event.parent,
        }
    }
}

#[derive(Debug)]
enum DispatchableContainer {
    Trace {
        timestamp: Duration,
        trace: DispatchableTrace,
    },
    End,
}

#[derive(Debug)]
enum DispatchableTrace {
    RegisterCallsite(DispatchableMetadata),
    Event(DispatchableEvent),
    NewSpan(DispatchableNewSpan),
    Enter(DispatchableSpanId),
    Exit(DispatchableSpanId),
    Close(DispatchableSpanId),
    Record(DispatchableRecordValues),
    FollowsFrom(DispatchableFollowsFrom),
}

#[derive(Debug)]
struct DispatchableMetadata(&'static Metadata<'static>);

impl DispatchableMetadata {
    fn into_inner(self) -> &'static Metadata<'static> {
        self.0
    }
}

#[derive(Debug)]
struct DispatchableEvent {
    metadata: &'static Metadata<'static>,
    fields: Vec<(String, String)>,
    parent: recording::Parent,
}

#[derive(Debug)]
struct DispatchableNewSpan {
    id: recording::SpanId,
    metadata: &'static Metadata<'static>,
    fields: Vec<(String, String)>,
    parent: recording::Parent,
}

#[derive(Debug)]
struct DispatchableSpanId(recording::SpanId);

impl DispatchableSpanId {
    fn into_inner(self) -> recording::SpanId {
        self.0
    }
}

#[derive(Debug)]
struct DispatchableFollowsFrom {
    cause_id: recording::SpanId,
    effect_id: recording::SpanId,
}

#[derive(Debug)]
pub(crate) struct DispatchableRecordValues {
    id: recording::SpanId,
    metadata: &'static Metadata<'static>,
    fields: Vec<(String, String)>,
}

struct ThreadDispatcher {
    rec_id: String,
    trace_rx: mpsc::Receiver<DispatchableContainer>,
    span_ids: Arc<Mutex<HashMap<recording::SpanId, MappedSpanId>>>,
}

impl ThreadDispatcher {
    fn run(self) {
        let rec_id = &self.rec_id;
        loop {
            match self.trace_rx.recv() {
                Ok(DispatchableContainer::Trace { timestamp, trace }) => {
                    self.dispatch(timestamp, trace);
                }
                Ok(DispatchableContainer::End) => break,
                Err(err) => {
                    println!("rec_id={rec_id}: Got error: {err}.");
                    break;
                }
            }
        }
    }

    fn dispatch(&self, timestamp: Duration, trace: DispatchableTrace) {
        let delay = timestamp.saturating_sub(SystemTime::now().duration_since(UNIX_EPOCH).unwrap());
        if !delay.is_zero() {
            thread::sleep(delay);
        }

        match trace {
            DispatchableTrace::RegisterCallsite(dis_metadata) => {
                let metadata = dis_metadata.into_inner();
                tracing::dispatcher::get_default(move |dispatch| {
                    dispatch.register_callsite(metadata);
                });
            }
            DispatchableTrace::Event(dis_event) => {
                tracing::dispatcher::get_default(move |dispatch| {
                    let enabled = dispatch.enabled(dis_event.metadata);
                    if enabled {
                        let values = create_field_values(dis_event.metadata, &dis_event.fields);
                        let proxy =
                            EventProxy::new(dispatch, dis_event.metadata, &dis_event.parent);
                        proxy.dispatch_values(values);
                    }
                });
            }
            DispatchableTrace::NewSpan(dis_new_span) => {
                tracing::dispatcher::get_default(move |dispatch| {
                    if !dispatch.enabled(dis_new_span.metadata) {
                        return;
                    }

                    let values = create_field_values(dis_new_span.metadata, &dis_new_span.fields);
                    let proxy =
                        NewSpanProxy::new(dispatch, dis_new_span.metadata, &dis_new_span.parent);
                    let span_id = proxy.dispatch_values(values);

                    // Store a mapping from the recorded span::Id to the one that `tracing` has given us
                    // during this replay. We will need to look up this mapping to replay traces that
                    // reference this new span by Id (enter, exit, ...).
                    {
                        let mut guard = self
                            .span_ids
                            .lock()
                            .expect("replay internal state has become corrupted.");

                        // TODO(hds): This should check that the entry is exactly Some(MappedSpanId::Pending) and nothing else.
                        let current_value = (*guard).get(&dis_new_span.id);
                        debug_assert!(
                            matches!((*guard).get(&dis_new_span.id), Some(MappedSpanId::Pending)),
                            "new span recorded span::Id should be Pending, but is {current_value:?}",
                        );
                        (*guard).insert(dis_new_span.id, MappedSpanId::Mapped(span_id));
                    }
                });
            }
            DispatchableTrace::Enter(dis_span_id) => {
                let span_id = self
                    .get_replay_span_id(dis_span_id.into_inner())
                    .expect("no replay span::Id found, is the recording complete?");
                tracing::dispatcher::get_default(|dispatch| dispatch.enter(&span_id));
            }
            DispatchableTrace::Exit(dis_span_id) => {
                let span_id = self
                    .get_replay_span_id(dis_span_id.into_inner())
                    .expect("no replay span::Id found, is the recording complete?");
                tracing::dispatcher::get_default(|dispatch| dispatch.exit(&span_id));
            }
            DispatchableTrace::Close(dis_span_id) => {
                let span_id = self
                    .get_replay_span_id(dis_span_id.into_inner())
                    .expect("no replay span::Id found, is the recording complete?");
                tracing::dispatcher::get_default(|dispatch| dispatch.try_close(span_id.clone()));
            }
            DispatchableTrace::Record(dis_record_values) => {
                let Some(span_id) = self.get_replay_span_id(dis_record_values.id) else {
                    return;
                };

                tracing::dispatcher::get_default(move |dispatch| {
                    let values =
                        create_field_values(dis_record_values.metadata, &dis_record_values.fields);
                    let proxy = RecordProxy::new(dispatch, dis_record_values.metadata, &span_id);
                    proxy.dispatch_values(values);
                });
            }
            DispatchableTrace::FollowsFrom(dis_follows_from) => {
                let Some(cause_span_id) = self.get_replay_span_id(dis_follows_from.cause_id) else {
                    return;
                };
                let Some(effect_span_id) = self.get_replay_span_id(dis_follows_from.effect_id)
                else {
                    return;
                };
                tracing::dispatcher::get_default(move |dispatch| {
                    dispatch.record_follows_from(&effect_span_id, &cause_span_id);
                });
            }
        }
    }

    fn get_replay_span_id(&self, rec_span_id: recording::SpanId) -> Option<span::Id> {
        loop {
            let guard = self
                .span_ids
                .lock()
                .expect("replay internal state has become corrupted.");

            match (*guard).get(&rec_span_id) {
                Some(MappedSpanId::Pending) => {} // Spin lock, it must be coming soon!
                Some(MappedSpanId::Mapped(span_id)) => break Some(span_id.clone()),
                None => break None,
            }
        }
    }
}

#[derive(Debug)]
struct ThreadDispatcherHandle {
    join_handle: JoinHandle<()>,
    trace_tx: mpsc::Sender<DispatchableContainer>,
}

fn create_field_values<'a>(
    metadata: &'static Metadata,
    rec_fields: &'a [(String, String)],
) -> Vec<(field::Field, Option<&'a dyn tracing::Value>)> {
    let fields = metadata.fields();
    rec_fields
        .iter()
        .filter_map(|(field_name, value)| {
            Some((fields.field(field_name)?, Some(value as &dyn field::Value)))
        })
        .collect()
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
