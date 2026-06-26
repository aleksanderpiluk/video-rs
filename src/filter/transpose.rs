use super::{Filter, InPadHandle, OutPadHandle, Registerable, Synthesizeable};

#[derive(Debug)]
pub struct TransposeFilter {
    id: String,
    dir: TransposeDir,
    passthrough: TransposePassthrough,
}

impl TransposeFilter {
    pub fn new(id: impl Into<String>, dir: TransposeDir) -> Self {
        Self {
            id: id.into(),
            dir,
            passthrough: TransposePassthrough::None,
        }
    }

    pub fn with_passthrough(mut self, passthrough: TransposePassthrough) -> Self {
        self.passthrough = passthrough;
        self
    }
}

impl Filter<TransposeFilterHandle> for TransposeFilter {}

impl Synthesizeable for TransposeFilter {
    fn synthesize(&self, ctx: &mut super::SynthesisContext) {
        let dir = match self.dir {
            TransposeDir::CClockFlip => "cclock_flip",
            TransposeDir::Clock => "clock",
            TransposeDir::CClock => "cclock",
            TransposeDir::ClockFlip => "clock_flip",
        };

        let passthrough = match self.passthrough {
            TransposePassthrough::None => "none",
            TransposePassthrough::Portrait => "portrait",
            TransposePassthrough::Landscape => "landscape",
        };

        ctx.add_filter("transpose", &self.id)
            .add_param("dir", dir)
            .add_param("passthrough", passthrough)
            .build();
    }
}

impl Registerable for TransposeFilter {
    type Handle = TransposeFilterHandle;

    fn register(self, ctx: &mut super::NodeContext) -> (Box<dyn Synthesizeable>, Self::Handle) {
        (
            Box::new(self),
            TransposeFilterHandle {
                input: ctx.new_in_pad(0),
                output: ctx.new_out_pad(0),
            },
        )
    }
}

pub struct TransposeFilterHandle {
    input: InPadHandle,
    output: OutPadHandle,
}

impl TransposeFilterHandle {
    pub fn input(&self) -> InPadHandle {
        self.input
    }

    pub fn output(&self) -> OutPadHandle {
        self.output
    }
}

#[derive(Debug)]
pub enum TransposeDir {
    CClockFlip,
    Clock,
    CClock,
    ClockFlip,
}

#[derive(Debug)]
pub enum TransposePassthrough {
    None,
    Portrait,
    Landscape,
}
