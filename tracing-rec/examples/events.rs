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

    for idx in 0..3 {
        tracing::trace!(idx, "Event in a loop");
    }
}
