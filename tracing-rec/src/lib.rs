use std::io::{stdout, Stdout, Write};

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
            id: value as *const _ as u64,
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
struct Event {
    fields: Vec<(&'static str, String)>,
    metadata: Metadata,
    parent: Parent,
}

impl From<&tracing::Event<'_>> for Event {
    fn from(value: &tracing::Event<'_>) -> Self {
        let mut event = Self {
            fields: Vec::new(),
            metadata: value.metadata().into(),
            parent: Parent::from(value),
        };
        value.record(&mut event);

        event
    }
}

impl Visit for Event {
    fn record_debug(&mut self, field: &tracing::field::Field, value: &dyn std::fmt::Debug) {
        self.fields.push((field.name(), format!("{value:?}")));
    }
}

#[derive(Debug, Serialize)]
struct NewSpan {
    id: SpanId,
    fields: Vec<(&'static str, String)>,
    metadata: Metadata,
    parent: Parent,
}

impl From<(&span::Attributes<'_>, &span::Id)> for NewSpan {
    fn from((attrs, id): (&span::Attributes<'_>, &span::Id)) -> Self {
        let mut new_span = Self {
            id: id.into(),
            fields: Vec::new(),
            metadata: attrs.metadata().into(),
            parent: Parent::from(attrs),
        };
        attrs.record(&mut new_span);

        new_span
    }
}

impl Visit for NewSpan {
    fn record_debug(&mut self, field: &tracing::field::Field, value: &dyn std::fmt::Debug) {
        self.fields.push((field.name(), format!("{value:?}")));
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
    fields: Vec<(&'static str, String)>,
}

impl From<(&span::Id, &span::Record<'_>)> for RecordValues {
    fn from((id, values): (&span::Id, &span::Record<'_>)) -> Self {
        let mut record_values = Self {
            id: id.into(),
            fields: Vec::new(),
        };
        values.record(&mut record_values);

        record_values
    }
}

impl Visit for RecordValues {
    fn record_debug(&mut self, field: &tracing::field::Field, value: &dyn std::fmt::Debug) {
        self.fields.push((field.name(), format!("{value:?}")));
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
    fn write_trace(&self, trace: &Trace) {
        serde_json::to_writer(&self.writer, &trace).expect("writing failed");
        writeln!(&self.writer).expect("writing failed");
    }
}

impl<S> tracing_subscriber::Layer<S> for Rec
where
    S: Subscriber,
{
    fn register_callsite(&self, metadata: &'static tracing::Metadata<'static>) -> Interest {
        let trace = Trace::RegisterCallsite(metadata.into());
        serde_json::to_writer(stdout(), &trace).expect("writing failed");
        stdout().write_all(b"\n").expect("writing failed");

        Interest::always()
    }

    fn on_new_span(
        &self,
        attrs: &span::Attributes<'_>,
        id: &span::Id,
        _ctx: tracing_subscriber::layer::Context<'_, S>,
    ) {
        let trace = Trace::NewSpan((attrs, id).into());
        self.write_trace(&trace);
    }

    fn on_record(
        &self,
        span: &span::Id,
        values: &span::Record<'_>,
        _ctx: tracing_subscriber::layer::Context<'_, S>,
    ) {
        let trace = Trace::Record((span, values).into());
        self.write_trace(&trace);
    }

    fn on_follows_from(
        &self,
        span: &span::Id,
        follows: &span::Id,
        _ctx: tracing_subscriber::layer::Context<'_, S>,
    ) {
        let trace = Trace::FollowsFrom(FollowsFrom::new(follows.into(), span.into()));
        self.write_trace(&trace);
    }

    fn on_event(
        &self,
        event: &tracing::Event<'_>,
        _ctx: tracing_subscriber::layer::Context<'_, S>,
    ) {
        let trace = Trace::Event(event.into());
        self.write_trace(&trace);
    }

    fn on_enter(&self, id: &span::Id, _ctx: tracing_subscriber::layer::Context<'_, S>) {
        let trace = Trace::Enter(id.into());
        self.write_trace(&trace);
    }

    fn on_exit(&self, id: &span::Id, _ctx: tracing_subscriber::layer::Context<'_, S>) {
        let trace = Trace::Exit(id.into());
        self.write_trace(&trace);
    }

    fn on_close(&self, id: span::Id, _ctx: tracing_subscriber::layer::Context<'_, S>) {
        let trace = Trace::Close((&id).into());
        self.write_trace(&trace);
    }
}
