use std::{
    collections::HashMap,
    mem::transmute,
    sync::{Arc, Mutex},
};

use tracing_core::{
    field,
    span::{self, Attributes},
    Event, Metadata,
};

pub fn crimes() {
    let metadata_store: Arc<Mutex<HashMap<u64, Metadata<'static>>>> =
        Arc::new(Mutex::new(HashMap::new()));

    let _span = tracing::info_span!("Span over them all", happy = true).entered();

    tracing::info!("This is an `info!` macro event");

    let span_id = new_span(3, "hand-span", metadata_store.clone());
    enter_span(&span_id);

    write_event(1, "This is a hand-rolled event", metadata_store.clone());
    write_event(
        1,
        "This is another hand-rolled event with the same metadata",
        metadata_store.clone(),
    );
    exit_span(&span_id);

    write_event(
        2,
        "This hand-rolled event has different metadata",
        metadata_store.clone(),
    );

    try_close_span(&span_id);
}

enum MetadataEntry {
    New(&'static Metadata<'static>),
    Existing(&'static Metadata<'static>),
}

fn metadata_or_create(
    metadata_id: u64,
    name: &'static str,
    store: &Arc<Mutex<HashMap<u64, Metadata<'static>>>>,
) -> MetadataEntry {
    let mut guard = store.lock().unwrap();

    let mut new_metadata = false;
    let metadata = (*guard).entry(metadata_id).or_insert_with(|| {
        new_metadata = true;
        let metadata: Metadata<'static> = make_metadata_with_level(
            name,
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
        let metadata = match metadata_or_create(metadata_id, "", &store.clone()) {
            MetadataEntry::New(metadata) => {
                dispatch.register_callsite(metadata);
                metadata
            }
            MetadataEntry::Existing(metadata) => metadata,
        };

        let enabled = dispatch.enabled(metadata);
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

fn new_span(
    metadata_id: u64,
    span_name: &'static str,
    store: Arc<Mutex<HashMap<u64, Metadata<'static>>>>,
) -> span::Id {
    tracing::dispatcher::get_default(move |dispatch| {
        let metadata = match metadata_or_create(metadata_id, span_name, &store.clone()) {
            MetadataEntry::New(metadata) => {
                dispatch.register_callsite(metadata);
                metadata
            }
            MetadataEntry::Existing(metadata) => metadata,
        };

        let fields = metadata.fields();
        let field = fields.field("field").unwrap();
        let values = [(&field, Some(&"field-value" as &dyn field::Value))];
        let value_set = metadata.fields().value_set(&values);

        let span_id = tracing::dispatcher::get_default(move |dispatch| {
            let span_attributes = Attributes::new(metadata, &value_set);
            dispatch.new_span(&span_attributes)
        });

        // let span = Span::new(metadata, &value_set);
        // span.id().unwrap()

        span_id
    })
}

fn enter_span(span_id: &span::Id) {
    tracing::dispatcher::get_default(|dispatch| dispatch.enter(span_id));
}

fn exit_span(span_id: &span::Id) {
    tracing::dispatcher::get_default(|dispatch| dispatch.exit(span_id));
}

fn try_close_span(span_id: &span::Id) -> bool {
    tracing::dispatcher::get_default(|dispatch| {
        let span_id = span_id.clone();
        dispatch.try_close(span_id)
    })
}

fn make_metadata_with_level(
    name: &'static str,
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
        name,
        "doing-crimes",
        level,
        Some("src/crimes.rs"),
        Some(line_number),
        None,
        tracing::field::FieldSet::new(&["message", "field"], tracing_core::identify_callsite!(cs)),
        tracing::metadata::Kind::EVENT,
    )
}
