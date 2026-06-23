use std::any::Any;

use crate::filter::{Registerable, Synthesizeable};

use super::{Filter, InPadHandle, OutPadHandle, SynthesisContext};

#[derive(Debug)]
pub struct SplitFilter {
    id: String,
    output_count: u32,
}

impl SplitFilter {
    pub fn new(id: impl Into<String>, output_count: u32) -> Self {
        Self {
            id: id.into(),
            output_count,
        }
    }
}

impl Filter<SplitFilterHandle> for SplitFilter {}

impl Synthesizeable for SplitFilter {
    fn synthesize(&self, ctx: &mut SynthesisContext) {
        ctx.add_filter("split", &self.id)
            .add_positional_param(&self.output_count.to_string())
            .build();
    }
}

impl Registerable for SplitFilter {
    type Handle = SplitFilterHandle;

    fn register(self, ctx: &mut super::NodeContext) -> (Box<dyn Synthesizeable>, Self::Handle) {
        let output_count = self.output_count;
        (
            Box::new(self),
            SplitFilterHandle {
                input: ctx.new_in_pad(0),
                outputs: (0..output_count).map(|i| ctx.new_out_pad(i)).collect(),
            },
        )
    }
}

pub struct SplitFilterHandle {
    input: InPadHandle,
    outputs: Vec<OutPadHandle>,
}

impl SplitFilterHandle {
    pub fn input(&self) -> InPadHandle {
        self.input
    }

    pub fn output(&self, index: usize) -> OutPadHandle {
        self.outputs[index]
    }
}
