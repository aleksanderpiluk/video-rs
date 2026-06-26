extern crate ffmpeg_next as ffmpeg;

use ffmpeg::sys::AVPixelFormat;
use ffmpeg::Rational as AvRational;

use super::{Filter, OutPadHandle, Registerable, Synthesizeable};
use crate::frame::PixelFormat;

#[derive(Debug)]
pub struct BufferFilter {
    id: String,
    width: u32,
    height: u32,
    pix_fmt: PixelFormat,
    time_base: AvRational,
    pixel_aspect: Option<AvRational>, // frame_rate: String,
}

impl BufferFilter {
    pub fn new(
        id: impl Into<String>,
        width: u32,
        height: u32,
        pix_fmt: PixelFormat,
        time_base: AvRational,
    ) -> Self {
        Self {
            id: id.into(),
            width,
            height,
            pix_fmt,
            time_base,
            pixel_aspect: None,
        }
    }

    pub fn pixel_aspect(mut self, pixel_aspect: AvRational) -> Self {
        self.pixel_aspect = Some(pixel_aspect);
        self
    }
}

impl Filter<BufferFilterHandle> for BufferFilter {}

impl Synthesizeable for BufferFilter {
    fn synthesize(&self, ctx: &mut super::SynthesisContext) {
        let mut filter = ctx.add_filter("buffer", &self.id)
            .add_param("width", &self.width.to_string())
            .add_param("height", &self.height.to_string())
            .add_param(
                "pix_fmt",
                &(AVPixelFormat::from(self.pix_fmt) as i32).to_string(),
            )
            .add_param("time_base", &self.time_base.to_string());
        
        if let Some(pixel_aspect) = self.pixel_aspect {
            filter = filter.add_param("sar", &pixel_aspect.to_string());
        }

        filter.build();
    }
}

impl Registerable for BufferFilter {
    type Handle = BufferFilterHandle;

    fn register(self, ctx: &mut super::NodeContext) -> (Box<dyn Synthesizeable>, Self::Handle) {
        (
            Box::new(self),
            BufferFilterHandle {
                output: ctx.new_out_pad(0),
            },
        )
    }
}

pub struct BufferFilterHandle {
    output: OutPadHandle,
}

impl BufferFilterHandle {
    pub fn output(&self) -> OutPadHandle {
        self.output
    }
}
