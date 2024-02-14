use tracing_subscriber::{fmt::format::FmtSpan, prelude::*};

fn main() {
    let layer = tracing_subscriber::fmt::Layer::default()
        .with_file(true)
        .with_line_number(true)
        .with_span_events(FmtSpan::FULL);
    tracing_subscriber::registry().with(layer).init();

    tracing_replay::crimes();
}
