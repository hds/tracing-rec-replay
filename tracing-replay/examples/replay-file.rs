use std::env;

use tracing_subscriber::{fmt::format::FmtSpan, prelude::*};

fn main() -> Result<(), String> {
    let layer = tracing_subscriber::fmt::Layer::default()
        .with_file(true)
        .with_line_number(true)
        .with_span_events(FmtSpan::FULL);
    tracing_subscriber::registry().with(layer).init();

    let Some(path) = env::args().skip(1).next() else {
        return Err(
            "error: no recording filename provided. usage: replay-file <recording_file>".into(),
        );
    };

    let mut replay = tracing_replay::Replay::new();
    let summary = replay
        .replay_file(&path)
        .map_err(|err| format!("failed to replay file: {path}, error: {err}."))?;
    println!(
        "Successully replayed, record count: {record_count}.",
        record_count = summary.record_count
    );

    Ok(())
}
