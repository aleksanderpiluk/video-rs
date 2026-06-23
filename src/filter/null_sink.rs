use crate::filter::{Filter, InPadHandle, Registerable, Synthesizeable};

#[derive(Debug)]
pub struct NullSinkFilter {
    id: String,
}

impl NullSinkFilter {
    pub fn new(id: impl Into<String>) -> Self {
        Self { id: id.into() }
    }
}

impl Filter<NullSinkFilterHandle> for NullSinkFilter {}

impl Synthesizeable for NullSinkFilter {
    fn synthesize(&self, graph: &mut super::SynthesisContext) {
        graph.add_filter("nullsink", &self.id).build();
    }
}

impl Registerable for NullSinkFilter {
    type Handle = NullSinkFilterHandle;

    fn register(self, ctx: &mut super::NodeContext) -> (Box<dyn Synthesizeable>, Self::Handle) {
        (
            Box::new(self),
            NullSinkFilterHandle {
                input: ctx.new_in_pad(0),
            },
        )
    }
}

pub struct NullSinkFilterHandle {
    input: InPadHandle,
}

impl NullSinkFilterHandle {
    pub fn input(&self) -> InPadHandle {
        self.input
    }
}
