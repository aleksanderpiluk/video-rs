extern crate ffmpeg_next as ffmpeg;
use crate::filter::{Registerable, Synthesizeable};

use super::{Filter, InPadHandle, NodeContext, OutPadHandle, SynthesisContext};

#[derive(Debug)]
pub struct RotateFilter {
    id: String,
    angle: Option<String>,
}

impl RotateFilter {
    pub fn new(id: impl Into<String>) -> Self {
        Self {
            id: id.into(),
            angle: None,
        }
    }

    pub fn with_angle(mut self, angle: impl Into<String>) -> Self {
        self.angle = Some(angle.into());
        self
    }
}

impl Filter<RotateFilterHandle> for RotateFilter {}

impl Synthesizeable for RotateFilter {
    fn synthesize(&self, ctx: &mut SynthesisContext) {
        ctx.add_filter("rotate", &self.id)
            .add_param(
                "angle",
                match &self.angle {
                    Some(angle) => angle,
                    None => "0",
                },
            )
            .build();
        // .add_param("angle", &self.angle.as_deref().unwrap())
    }

    // fn synthesize(&self, graph: &AvFilterGraph) -> String {
    //     let args = format!("rotate=angle={}", self.angle);
    //     let filter = graph.add(&ffmpeg::filter::find("rotate").unwrap(), "name", &args).unwrap();

    //     filter.link(0, dst, dstpad);
    // }
}

impl Registerable for RotateFilter {
    type Handle = RotateFilterHandle;

    fn register(self, ctx: &mut NodeContext) -> (Box<dyn Synthesizeable>, Self::Handle) {
        (
            Box::new(self),
            RotateFilterHandle {
                input: ctx.new_in_pad(0),
                output: ctx.new_out_pad(0),
            },
        )
    }

    // fn register(self, ctx: &mut NodeContext) ->  {

    // }
}

pub struct RotateFilterHandle {
    input: InPadHandle,
    output: OutPadHandle,
}

impl RotateFilterHandle {
    pub fn input(&self) -> InPadHandle {
        self.input
    }

    pub fn output(&self) -> OutPadHandle {
        self.output
    }
}
