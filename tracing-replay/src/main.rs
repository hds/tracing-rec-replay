use tracing_subscriber::prelude::*;

fn main() {
    let layer = tracing_subscriber::fmt::Layer::default()
        .with_file(true)
        .with_line_number(true);
    tracing_subscriber::registry().with(layer).init();

    tracing_replay::crimes();
}
