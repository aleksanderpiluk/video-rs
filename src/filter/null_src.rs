use crate::filter::{Filter, OutPadHandle, Registerable, Synthesizeable};

#[derive(Debug)]
pub struct NullSrcFilter {
    id: String,
}

impl NullSrcFilter {
    pub fn new(id: impl Into<String>) -> Self {
        Self { id: id.into() }
    }
}

impl Filter<NullSrcFilterHandle> for NullSrcFilter {}

impl Synthesizeable for NullSrcFilter {
    fn synthesize(&self, graph: &mut super::SynthesisContext) {
        graph.add_filter("nullsrc", &self.id).build();
    }
}

impl Registerable for NullSrcFilter {
    type Handle = NullSrcFilterHandle;

    fn register(self, ctx: &mut super::NodeContext) -> (Box<dyn Synthesizeable>, Self::Handle) {
        (
            Box::new(self),
            NullSrcFilterHandle {
                output: ctx.new_out_pad(0),
            },
        )
    }
}

pub struct NullSrcFilterHandle {
    output: OutPadHandle,
}

impl NullSrcFilterHandle {
    pub fn output(&self) -> OutPadHandle {
        self.output
    }
}
