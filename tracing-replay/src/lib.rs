use std::{
    collections::HashMap,
    mem::transmute,
    sync::{Arc, Mutex},
};

use tracing_core::{field, Event, Metadata};

pub fn crimes() {
    let metadata_store: Arc<Mutex<HashMap<u64, Metadata<'static>>>> =
        Arc::new(Mutex::new(HashMap::new()));

    tracing::info!("This is an `info!` macro event");

    write_event(1, "This is a hand-rolled event", metadata_store.clone());
    write_event(
        1,
        "This is another hand-rolled event, but with the same metadata",
        metadata_store.clone(),
    );
    write_event(
        2,
        "This hand-rolled event has different metadata",
        metadata_store.clone(),
    );
}

enum MetadataEntry {
    New(&'static Metadata<'static>),
    Existing(&'static Metadata<'static>),
}

fn metadata_or_create(
    metadata_id: u64,
    store: Arc<Mutex<HashMap<u64, Metadata<'static>>>>,
) -> MetadataEntry {
    let mut guard = store.lock().unwrap();

    let mut new_metadata = false;
    let metadata = (*guard).entry(metadata_id).or_insert_with(|| {
        new_metadata = true;
        let metadata: Metadata<'static> = make_metadata_with_level(
            tracing_core::Level::INFO,
            TryInto::<u32>::try_into(metadata_id).unwrap() * 5 + 100,
        );
        metadata
    });

    let metadata: &'static Metadata = unsafe {
        // I promise to never remove this item from the HashMap. This is not a real safety
        // guarantee.
        transmute::<&'_ Metadata<'static>, &'static Metadata<'static>>(metadata)
    };

    if new_metadata {
        MetadataEntry::New(metadata)
    } else {
        MetadataEntry::Existing(metadata)
    }
}

fn write_event(metadata_id: u64, msg: &str, store: Arc<Mutex<HashMap<u64, Metadata<'static>>>>) {
    let _enabled = tracing::dispatcher::get_default(move |dispatch| {
        let metadata = match metadata_or_create(metadata_id, store.clone()) {
            MetadataEntry::New(metadata) => {
                dispatch.register_callsite(metadata);
                metadata
            }
            MetadataEntry::Existing(metadata) => metadata,
        };

        let enabled = dispatch.enabled(&metadata);
        {
            let fields = metadata.fields();
            let message_field = fields.field("message").unwrap();
            let values = [(&message_field, Some(&msg as &dyn field::Value))];
            let value_set = metadata.fields().value_set(&values);
            let event = Event::new(metadata, &value_set);
            dispatch.event(&event);
        }

        enabled
    });
}

fn make_metadata_with_level(
    level: tracing_core::Level,
    line_number: u32,
) -> tracing::Metadata<'static> {
    struct Cs;
    impl Cs {
        fn new() -> Self {
            Cs
        }
    }
    impl tracing_core::Callsite for Cs {
        fn set_interest(&self, _interest: tracing_core::Interest) {}
        fn metadata(&self) -> &tracing_core::Metadata<'_> {
            unimplemented!()
        }
    }

    let cs: &'static Cs = Box::leak(Box::new(Cs::new()));
    tracing::Metadata::new(
        "",
        "doing-crimes",
        level,
        Some("src/crimes.rs"),
        Some(line_number),
        None,
        tracing::field::FieldSet::new(&["message"], tracing_core::identify_callsite!(cs)),
        tracing::metadata::Kind::EVENT,
    )
}
