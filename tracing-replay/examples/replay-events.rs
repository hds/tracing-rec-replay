use tracing_replay::ReplaySummary;
use tracing_subscriber::{fmt::format::FmtSpan, prelude::*};

fn main() {
    let layer = tracing_subscriber::fmt::Layer::default()
        .with_file(true)
        .with_line_number(true)
        .with_span_events(FmtSpan::FULL);
    tracing_subscriber::registry().with(layer).init();

    let mut replay = tracing_replay::Replay::new();
    match replay.replay_file("sample-data/events.tracing") {
        Ok(ReplaySummary { record_count, .. }) => {
            println!("Successully replayed, record count: {record_count}.")
        }
        Err(err) => println!("Failed to replay, error: {err}"),
    }
}
