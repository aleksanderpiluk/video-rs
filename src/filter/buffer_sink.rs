use super::{Filter, InPadHandle, Registerable, Synthesizeable};

#[derive(Debug)]
pub struct BufferSinkFilter {
    id: String,
}

impl BufferSinkFilter {
    pub fn new(id: impl Into<String>) -> Self {
        Self { id: id.into() }
    }
}

impl Filter<BufferSinkFilterHandle> for BufferSinkFilter {}

impl Synthesizeable for BufferSinkFilter {
    fn synthesize(&self, ctx: &mut super::SynthesisContext) {
        ctx.add_filter("buffersink", &self.id).build();
    }
}

impl Registerable for BufferSinkFilter {
    type Handle = BufferSinkFilterHandle;

    fn register(self, ctx: &mut super::NodeContext) -> (Box<dyn Synthesizeable>, Self::Handle) {
        (
            Box::new(self),
            BufferSinkFilterHandle {
                input: ctx.new_in_pad(0),
            },
        )
    }
}

pub struct BufferSinkFilterHandle {
    input: InPadHandle,
}

impl BufferSinkFilterHandle {
    pub fn input(&self) -> InPadHandle {
        self.input
    }
}
