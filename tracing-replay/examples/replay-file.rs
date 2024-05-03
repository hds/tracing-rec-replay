use std::{env, error};

use tracing_subscriber::{fmt::format::FmtSpan, prelude::*};

fn main() -> Result<(), Box<dyn error::Error>> {
    let layer = tracing_subscriber::fmt::Layer::default()
        .with_file(true)
        .with_line_number(true)
        .with_thread_ids(true)
        .with_thread_names(true)
        .with_span_events(FmtSpan::FULL);
    tracing_subscriber::registry().with(layer).init();

    let Some(path) = env::args().nth(1) else {
        return Err(
            "error: no recording filename provided. usage: replay-file <recording_file>".into(),
        );
    };

    let mut replay = tracing_replay::Replay::new();
    let summary = replay
        .replay_file(&path)
        .map_err(|err| format!("failed to replay file: {path}, error: {err}."))?;
    replay.close()?;
    println!(
        "Successully replayed, record count: {record_count}.",
        record_count = summary.record_count
    );

    Ok(())
}
