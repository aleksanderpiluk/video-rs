use super::{Filter, InPadHandle, OutPadHandle, Registerable, Synthesizeable};

#[derive(Debug)]
pub struct HFlipFilter {
    id: String,
}

impl HFlipFilter {
    pub fn new(id: impl Into<String>) -> Self {
        Self { id: id.into() }
    }
}

impl Filter<HFlipFilterHandle> for HFlipFilter {}

impl Synthesizeable for HFlipFilter {
    fn synthesize(&self, ctx: &mut super::SynthesisContext) {
        ctx.add_filter("hflip", &self.id).build();
    }
}

impl Registerable for HFlipFilter {
    type Handle = HFlipFilterHandle;

    fn register(self, ctx: &mut super::NodeContext) -> (Box<dyn Synthesizeable>, Self::Handle) {
        (
            Box::new(self),
            HFlipFilterHandle {
                input: ctx.new_in_pad(0),
                output: ctx.new_out_pad(0),
            },
        )
    }
}

pub struct HFlipFilterHandle {
    input: InPadHandle,
    output: OutPadHandle,
}

impl HFlipFilterHandle {
    pub fn input(&self) -> InPadHandle {
        self.input
    }

    pub fn output(&self) -> OutPadHandle {
        self.output
    }
}
