use tracing_subscriber::prelude::*;

fn main() {
    tracing_subscriber::registry()
        .with(tracing_rec::rec_layer())
        .init();

    tracing::info!("I am an info event!");

    tracing::error!(parent: None, broken = true, "I have a field!");

    let span: tracing::Span = tracing::info_span!("span");
    span.in_scope(|| {
        tracing::debug!(working = true, "Message with interpolated value: {}", 42);
    });

    tracing::warn!(parent: span, "Event with explicit parent");

    tracing::info!(
        field00 = 0,
        field01 = 1,
        field02 = 2,
        field03 = 3,
        field04 = 4,
        field05 = 5,
        field06 = 6,
        field07 = 7,
        field08 = 8,
        field09 = 9,
        field10 = 10,
        field11 = 11,
        field12 = 12,
        field13 = 13,
        field14 = 14,
        field15 = 15,
        field16 = 16,
        field17 = 17,
        field18 = 18,
        field19 = 19,
        field20 = 20,
        field21 = 21,
        field22 = 22,
        field23 = 23,
        field24 = 24,
        field25 = 25,
        field26 = 26,
        field27 = 27,
        field28 = 28,
        field29 = 29,
        field30 = 30,
        field31 = 31,
        "many fields"
    );

    for idx in 0..3 {
        tracing::trace!(idx, "Event in a loop");
    }
}
