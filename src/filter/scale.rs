use crate::filter::{Filter, InPadHandle, OutPadHandle, Registerable, Synthesizeable};

#[derive(Debug)]
pub struct ScaleFilter {
    id: String,
    width: String,
    height: String,
}

impl ScaleFilter {
    pub fn new(id: impl Into<String>, width: impl Into<String>, height: impl Into<String>) -> Self {
        Self {
            id: id.into(),
            width: width.into(),
            height: height.into(),
        }
    }
}

impl Filter<ScaleFilterHandle> for ScaleFilter {}

impl Synthesizeable for ScaleFilter {
    fn synthesize(&self, ctx: &mut super::SynthesisContext) {
        ctx.add_filter("scale", &self.id)
            .add_param("width", &self.width)
            .add_param("height", &self.height)
            .build();
    }
}

impl Registerable for ScaleFilter {
    type Handle = ScaleFilterHandle;

    fn register(self, ctx: &mut super::NodeContext) -> (Box<dyn Synthesizeable>, Self::Handle) {
        (
            Box::new(self),
            ScaleFilterHandle {
                input: ctx.new_in_pad(0),
                output: ctx.new_out_pad(0),
            },
        )
    }
}

pub struct ScaleFilterHandle {
    input: InPadHandle,
    output: OutPadHandle,
}

impl ScaleFilterHandle {
    pub fn input(&self) -> InPadHandle {
        self.input
    }

    pub fn output(&self) -> OutPadHandle {
        self.output
    }
}
