use serde::Deserialize;
use tracing::field;

#[derive(Debug, Deserialize)]
pub(crate) struct TraceRecord {
    pub(crate) meta: RecordMeta,
    pub(crate) trace: Trace,
}

#[derive(Debug, Deserialize)]
pub(crate) struct RecordMeta {
    pub(crate) timestamp_s: u64,
    pub(crate) timestamp_subsec_us: u32,
    pub(crate) thread_id: String,
    pub(crate) thread_name: Option<String>,
}

#[derive(Debug, Deserialize)]
pub(crate) enum Trace {
    RegisterCallsite(Metadata),
    Event(Event),
    NewSpan(NewSpan),
    Enter(SpanId),
    Exit(SpanId),
    Close(SpanId),
    Record(RecordValues),
    FollowsFrom(FollowsFrom),
}

#[derive(Debug, Deserialize)]
pub(crate) enum Level {
    Trace,
    Debug,
    Info,
    Warn,
    Error,
}

impl From<Level> for tracing::Level {
    fn from(value: Level) -> Self {
        match value {
            Level::Trace => Self::TRACE,
            Level::Debug => Self::DEBUG,
            Level::Info => Self::INFO,
            Level::Warn => Self::WARN,
            Level::Error => Self::ERROR,
        }
    }
}

#[derive(Debug, Deserialize)]
pub(crate) enum Kind {
    Span,
    Event,
}

impl From<Kind> for tracing::metadata::Kind {
    fn from(value: Kind) -> Self {
        match value {
            Kind::Event => Self::EVENT,
            Kind::Span => Self::SPAN,
        }
    }
}

#[derive(Debug, Deserialize)]
pub(crate) struct Metadata {
    pub(crate) id: u64,
    pub(crate) name: String,
    pub(crate) target: String,
    pub(crate) level: Level,
    pub(crate) module_path: Option<String>,
    pub(crate) file: Option<String>,
    pub(crate) line: Option<u32>,
    pub(crate) fields: Vec<String>,
    pub(crate) kind: Kind,
}

#[derive(Debug, Deserialize)]
pub(crate) enum Parent {
    /// The new span will be a root span.
    Root,
    /// The new span will be rooted in the current span.
    Current,
    /// The new span has an explicitly-specified parent.
    Explicit(u64),
}

#[derive(Debug, Deserialize)]
pub(crate) struct Field {
    pub(crate) name: String,
    pub(crate) value: FieldValue,
}

#[derive(Debug, Deserialize)]
pub(crate) enum FieldValue {
    Debug(String),
    F64(f64),
    I64(i64),
    U64(u64),
    I128(i128),
    U128(u128),
    Bool(bool),
    Str(String),
}

impl<'a> From<&'a FieldValue> for &'a dyn field::Value {
    fn from(value: &'a FieldValue) -> Self {
        match value {
            FieldValue::Debug(val) => val as &dyn field::Value,
            FieldValue::F64(val) => val as &dyn field::Value,
            FieldValue::I64(val) => val as &dyn field::Value,
            FieldValue::U64(val) => val as &dyn field::Value,
            FieldValue::I128(val) => val as &dyn field::Value,
            FieldValue::U128(val) => val as &dyn field::Value,
            FieldValue::Bool(val) => val as &dyn field::Value,
            FieldValue::Str(val) => val as &dyn field::Value,
        }
    }
}

#[derive(Debug, Deserialize)]
pub(crate) struct Event {
    pub(crate) fields: Vec<Field>,
    pub(crate) metadata: Metadata,
    pub(crate) parent: Parent,
}

#[derive(Debug, Deserialize)]
pub(crate) struct NewSpan {
    pub(crate) id: SpanId,
    pub(crate) fields: Vec<Field>,
    pub(crate) metadata: Metadata,
    pub(crate) parent: Parent,
}

#[derive(Clone, Copy, Debug, Deserialize, Eq, PartialEq, Hash)]
pub(crate) struct SpanId(u64);

#[derive(Debug, Deserialize)]
pub(crate) struct RecordValues {
    pub(crate) id: SpanId,
    pub(crate) fields: Vec<Field>,
}

#[derive(Debug, Deserialize)]
pub(crate) struct FollowsFrom {
    pub(crate) cause_id: SpanId,
    pub(crate) effect_id: SpanId,
}
