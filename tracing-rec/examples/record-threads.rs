use std::{thread, time::Duration};

use tracing_subscriber::prelude::*;

fn main() {
    tracing_subscriber::registry()
        .with(tracing_rec::rec_layer())
        .init();

    let jh = std::thread::Builder::new()
        .name("other-thread".into())
        .spawn(move || {
            let span = tracing::info_span!("other-span");
            thread::sleep(Duration::from_millis(100));
            {
                let _entered = span.enter();
                tracing::info!("Hi there, it's a bit later");
                thread::sleep(Duration::from_millis(100));
            }
        })
        .unwrap();

    thread::sleep(Duration::from_millis(50));
    let span: tracing::Span = tracing::info_span!("span");

    {
        let _entered = span.enter();

        thread::sleep(Duration::from_millis(100));

        tracing::info!("I am an info event in which thread?");
    }

    _ = jh.join();
}
