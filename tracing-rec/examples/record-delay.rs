use std::{thread, time::Duration};

use tracing_subscriber::prelude::*;

fn main() {
    tracing_subscriber::registry()
        .with(tracing_rec::rec_layer())
        .init();

    tracing::info!("event before delay");

    thread::sleep(Duration::from_secs(2));

    tracing::info!("event after delay");
}
