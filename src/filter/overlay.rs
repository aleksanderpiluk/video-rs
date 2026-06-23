use crate::filter::{Registerable, Synthesizeable};

use super::{Filter, InPadHandle, OutPadHandle, SynthesisContext};

#[derive(Debug)]
pub struct OverlayFilter {
    id: String,
}

impl OverlayFilter {
    pub fn new(id: impl Into<String>) -> Self {
        Self { id: id.into() }
    }
}

impl Filter<OverlayFilterHandle> for OverlayFilter {}

impl Synthesizeable for OverlayFilter {
    fn synthesize(&self, ctx: &mut SynthesisContext) {
        ctx.add_filter("overlay", &self.id).build();
    }
}

impl Registerable for OverlayFilter {
    type Handle = OverlayFilterHandle;

    fn register(self, ctx: &mut super::NodeContext) -> (Box<dyn Synthesizeable>, Self::Handle) {
        (
            Box::new(self),
            OverlayFilterHandle {
                main: ctx.new_in_pad(0),
                overlay: ctx.new_in_pad(1),
                output: ctx.new_out_pad(0),
            },
        )
    }
}

pub struct OverlayFilterHandle {
    main: InPadHandle,
    overlay: InPadHandle,
    output: OutPadHandle,
}

impl OverlayFilterHandle {
    pub fn main_input(&self) -> InPadHandle {
        self.main
    }

    pub fn overlay_input(&self) -> InPadHandle {
        self.overlay
    }

    pub fn output(&self) -> OutPadHandle {
        self.output
    }
}
