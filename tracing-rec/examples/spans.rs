use tracing_subscriber::prelude::*;

fn main() {
    tracing_subscriber::registry()
        .with(tracing_rec::rec_layer())
        .init();

    let info_span = tracing::info_span!("info-span", value_will_change = 15);
    let _info_guard = info_span.enter();

    let enter_later = tracing::error_span!("enter-later");
    enter_later.follows_from(&info_span);

    let _later_guard = {
        let _direct_guard = tracing::warn_span!("direct-enter", wonderful = 42).entered();
        tracing::info!("an event");

        let later_guard = enter_later.enter();
        tracing::info!("I am an info event!");

        later_guard
    };

    loopy(3);

    info_span.record("value_will_change", 23);
}

#[tracing::instrument]
fn loopy(len: usize) {
    let enter_exit_span = tracing::debug_span!("enter-and-exit");
    for idx in 0..len {
        let _guard = enter_exit_span.enter();
        tracing::trace!(idx, "Event in a loop");
    }
}
