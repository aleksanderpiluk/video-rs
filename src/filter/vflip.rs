use super::{Filter, InPadHandle, OutPadHandle, Registerable, Synthesizeable};

#[derive(Debug)]
pub struct VFlipFilter {
    id: String,
}

impl VFlipFilter {
    pub fn new(id: impl Into<String>) -> Self {
        Self { id: id.into() }
    }
}

impl Filter<VFlipFilterHandle> for VFlipFilter {}

impl Synthesizeable for VFlipFilter {
    fn synthesize(&self, ctx: &mut super::SynthesisContext) {
        ctx.add_filter("vflip", &self.id).build();
    }
}

impl Registerable for VFlipFilter {
    type Handle = VFlipFilterHandle;

    fn register(self, ctx: &mut super::NodeContext) -> (Box<dyn Synthesizeable>, Self::Handle) {
        (
            Box::new(self),
            VFlipFilterHandle {
                input: ctx.new_in_pad(0),
                output: ctx.new_out_pad(0),
            },
        )
    }
}

pub struct VFlipFilterHandle {
    input: InPadHandle,
    output: OutPadHandle,
}
