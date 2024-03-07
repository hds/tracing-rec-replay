pub(crate) struct Cs {
    _id: u64,
}

impl Cs {
    pub(crate) fn new(id: u64) -> Self {
        Cs { _id: id }
    }
}

impl tracing_core::Callsite for Cs {
    fn set_interest(&self, _interest: tracing_core::Interest) {}
    fn metadata(&self) -> &tracing_core::Metadata<'_> {
        // FIXME(hds): When is this even called?
        unimplemented!()
    }
}
