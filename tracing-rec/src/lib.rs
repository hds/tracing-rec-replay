use std::{
    io::{stdout, Stdout, Write},
    time::{SystemTime, UNIX_EPOCH},
};

use serde::Serialize;
use tracing::{field::Visit, span, subscriber::Interest, Subscriber};

pub struct Rec {
    writer: Stdout,
}

#[must_use]
pub fn rec_layer() -> Rec {
    Rec { writer: stdout() }
}

#[derive(Debug, Serialize)]
struct TraceRecord {
    meta: RecordMeta,
    trace: Trace,
}

impl TraceRecord {
    fn implicit(trace: Trace) -> Self {
        Self {
            meta: RecordMeta::new(),
            trace,
        }
    }
}

#[derive(Debug, Serialize)]
struct RecordMeta {
    timestamp_s: u64,
    timestamp_subsec_us: u32,
    thread_id: String,
    thread_name: Option<String>,
}

impl RecordMeta {
    fn new() -> Self {
        let timestamp = SystemTime::now().duration_since(UNIX_EPOCH).unwrap();
        let thread = std::thread::current();

        Self {
            timestamp_s: timestamp.as_secs(),
            timestamp_subsec_us: timestamp.subsec_micros(),
            thread_id: format!("{:?}", thread.id()),
            thread_name: thread.name().map(Into::into),
        }
    }
}

#[derive(Debug, Serialize)]
enum Trace {
    RegisterCallsite(Metadata),
    Event(Event),
    NewSpan(NewSpan),
    Enter(SpanId),
    Exit(SpanId),
    Close(SpanId),
    Record(RecordValues),
    FollowsFrom(FollowsFrom),
}

#[derive(Debug, Serialize)]
enum Level {
    Trace,
    Debug,
    Info,
    Warn,
    Error,
}

impl From<&tracing::Level> for Level {
    fn from(value: &tracing::Level) -> Self {
        match *value {
            tracing::Level::TRACE => Level::Trace,
            tracing::Level::DEBUG => Level::Debug,
            tracing::Level::INFO => Level::Info,
            tracing::Level::WARN => Level::Warn,
            tracing::Level::ERROR => Level::Error,
        }
    }
}

#[derive(Debug, Serialize)]
enum Kind {
    Span,
    Event,
}

impl From<&'static tracing::Metadata<'static>> for Kind {
    fn from(value: &'static tracing::Metadata<'static>) -> Self {
        if value.is_event() {
            Self::Event
        } else {
            debug_assert!(
                value.is_span(),
                "either is_event() or is_span() should be true",
            );
            Self::Span
        }
    }
}

#[derive(Debug, Serialize)]
struct Metadata {
    id: u64,
    name: &'static str,
    target: &'static str,
    level: Level,
    module_path: Option<&'static str>,
    file: Option<&'static str>,
    line: Option<u32>,
    fields: Vec<&'static str>,
    kind: Kind,
}

impl From<&'static tracing::Metadata<'static>> for Metadata {
    fn from(value: &'static tracing::Metadata<'static>) -> Self {
        Self {
            id: std::ptr::from_ref(value) as u64,
            name: value.name(),
            target: value.target(),
            level: value.level().into(),
            module_path: value.module_path(),
            file: value.file(),
            line: value.line(),
            fields: value.fields().iter().map(|f| f.name()).collect(),
            kind: Kind::from(value),
        }
    }
}

#[derive(Debug, Serialize)]
enum Parent {
    /// The new span will be a root span.
    Root,
    /// The new span will be rooted in the current span.
    Current,
    /// The new span has an explicitly-specified parent.
    Explicit(u64),
}

impl From<&tracing::Event<'_>> for Parent {
    fn from(value: &tracing::Event<'_>) -> Self {
        if value.is_root() {
            Self::Root
        } else if value.is_contextual() {
            Self::Current
        } else {
            Self::Explicit(
                value
                    .parent()
                    .expect("a span that isn't root or contextual should have an explicit Id")
                    .into_u64(),
            )
        }
    }
}

impl From<&span::Attributes<'_>> for Parent {
    fn from(value: &span::Attributes<'_>) -> Self {
        if value.is_root() {
            Self::Root
        } else if value.is_contextual() {
            Self::Current
        } else {
            Self::Explicit(
                value
                    .parent()
                    .expect("a span that isn't root or contextual should have an explicit Id")
                    .into_u64(),
            )
        }
    }
}

#[derive(Debug, Serialize)]
struct Fields {
    inner: Vec<Field>,
}

impl Fields {
    fn new() -> Self {
        Self { inner: Vec::new() }
    }
}

impl Visit for Fields {
    fn record_debug(&mut self, field: &tracing::field::Field, value: &dyn std::fmt::Debug) {
        self.inner.push(Field::new(
            field.name(),
            FieldValue::Debug(format!("{value:?}")),
        ));
    }

    fn record_f64(&mut self, field: &tracing::field::Field, value: f64) {
        self.inner
            .push(Field::new(field.name(), FieldValue::F64(value)));
    }

    fn record_i64(&mut self, field: &tracing::field::Field, value: i64) {
        self.inner
            .push(Field::new(field.name(), FieldValue::I64(value)));
    }

    fn record_u64(&mut self, field: &tracing::field::Field, value: u64) {
        self.inner
            .push(Field::new(field.name(), FieldValue::U64(value)));
    }

    fn record_i128(&mut self, field: &tracing::field::Field, value: i128) {
        self.inner
            .push(Field::new(field.name(), FieldValue::I128(value)));
    }

    fn record_u128(&mut self, field: &tracing::field::Field, value: u128) {
        self.inner
            .push(Field::new(field.name(), FieldValue::U128(value)));
    }

    fn record_bool(&mut self, field: &tracing::field::Field, value: bool) {
        self.inner
            .push(Field::new(field.name(), FieldValue::Bool(value)));
    }

    fn record_str(&mut self, field: &tracing::field::Field, value: &str) {
        self.inner
            .push(Field::new(field.name(), FieldValue::Str(value.into())));
    }
}

#[derive(Debug, Serialize)]
struct Field {
    name: &'static str,
    value: FieldValue,
}

impl Field {
    fn new(name: &'static str, value: FieldValue) -> Self {
        Self { name, value }
    }
}

#[derive(Debug, Serialize)]
enum FieldValue {
    Debug(String),
    F64(f64),
    I64(i64),
    U64(u64),
    I128(i128),
    U128(u128),
    Bool(bool),
    Str(String),
    // TODO(hds): add variants for Value and Error
}

#[derive(Debug, Serialize)]
struct Event {
    fields: Vec<Field>,
    metadata: Metadata,
    parent: Parent,
}

impl From<&tracing::Event<'_>> for Event {
    fn from(value: &tracing::Event<'_>) -> Self {
        let mut fields = Fields::new();
        value.record(&mut fields);

        Self {
            fields: fields.inner,
            metadata: value.metadata().into(),
            parent: Parent::from(value),
        }
    }
}

#[derive(Debug, Serialize)]
struct NewSpan {
    id: SpanId,
    fields: Vec<Field>,
    metadata: Metadata,
    parent: Parent,
}

impl From<(&span::Attributes<'_>, &span::Id)> for NewSpan {
    fn from((attrs, id): (&span::Attributes<'_>, &span::Id)) -> Self {
        let mut fields = Fields::new();
        attrs.record(&mut fields);

        Self {
            id: id.into(),
            fields: fields.inner,
            metadata: attrs.metadata().into(),
            parent: Parent::from(attrs),
        }
    }
}

#[derive(Debug, Serialize)]
struct SpanId(u64);

impl From<&span::Id> for SpanId {
    fn from(value: &span::Id) -> Self {
        Self(value.into_u64())
    }
}

#[derive(Debug, Serialize)]
struct RecordValues {
    id: SpanId,
    fields: Vec<Field>,
}

impl From<(&span::Id, &span::Record<'_>)> for RecordValues {
    fn from((id, values): (&span::Id, &span::Record<'_>)) -> Self {
        let mut fields = Fields::new();
        values.record(&mut fields);

        Self {
            id: id.into(),
            fields: fields.inner,
        }
    }
}

#[derive(Debug, Serialize)]
struct FollowsFrom {
    cause_id: SpanId,
    effect_id: SpanId,
}

impl FollowsFrom {
    fn new(cause_id: SpanId, effect_id: SpanId) -> Self {
        Self {
            cause_id,
            effect_id,
        }
    }
}

impl Rec {
    fn write_trace(&self, trace_record: &TraceRecord) {
        serde_json::to_writer(&self.writer, &trace_record).expect("writing failed");
        writeln!(&self.writer).expect("writing failed");
    }
}

impl<S> tracing_subscriber::Layer<S> for Rec
where
    S: Subscriber,
{
    fn register_callsite(&self, metadata: &'static tracing::Metadata<'static>) -> Interest {
        let trace = Trace::RegisterCallsite(metadata.into());
        self.write_trace(&TraceRecord::implicit(trace));

        Interest::always()
    }

    fn on_new_span(
        &self,
        attrs: &span::Attributes<'_>,
        id: &span::Id,
        _ctx: tracing_subscriber::layer::Context<'_, S>,
    ) {
        let trace = Trace::NewSpan((attrs, id).into());
        self.write_trace(&TraceRecord::implicit(trace));
    }

    fn on_record(
        &self,
        span: &span::Id,
        values: &span::Record<'_>,
        _ctx: tracing_subscriber::layer::Context<'_, S>,
    ) {
        let trace = Trace::Record((span, values).into());
        self.write_trace(&TraceRecord::implicit(trace));
    }

    fn on_follows_from(
        &self,
        span: &span::Id,
        follows: &span::Id,
        _ctx: tracing_subscriber::layer::Context<'_, S>,
    ) {
        let trace = Trace::FollowsFrom(FollowsFrom::new(follows.into(), span.into()));
        self.write_trace(&TraceRecord::implicit(trace));
    }

    fn on_event(
        &self,
        event: &tracing::Event<'_>,
        _ctx: tracing_subscriber::layer::Context<'_, S>,
    ) {
        let trace = Trace::Event(event.into());
        self.write_trace(&TraceRecord::implicit(trace));
    }

    fn on_enter(&self, id: &span::Id, _ctx: tracing_subscriber::layer::Context<'_, S>) {
        let trace = Trace::Enter(id.into());
        self.write_trace(&TraceRecord::implicit(trace));
    }

    fn on_exit(&self, id: &span::Id, _ctx: tracing_subscriber::layer::Context<'_, S>) {
        let trace = Trace::Exit(id.into());
        self.write_trace(&TraceRecord::implicit(trace));
    }

    fn on_close(&self, id: span::Id, _ctx: tracing_subscriber::layer::Context<'_, S>) {
        let trace = Trace::Close((&id).into());
        self.write_trace(&TraceRecord::implicit(trace));
    }
}
