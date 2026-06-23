extern crate ffmpeg_next as ffmpeg;

use bytemuck::cast_slice;
use ffmpeg_next::ffi::av_display_rotation_get;

use crate::error::Error;
use crate::filter::{
    BufferFilter, BufferSinkFilter, FilterGraphBuilder, HFlipFilter, RotateFilter, TransposeDir,
    TransposeFilter, VFlipFilter,
};
use crate::frame::{RawFrame, FRAME_PIXEL_FORMAT};
use crate::{filter::FilterGraph, Decoder};

type Result<T> = std::result::Result<T, Error>;

pub struct VideoNormalizer {
    decoder: Decoder,
    // filter_graph: FilterGraph,
}

impl VideoNormalizer {
    pub fn new(decoder: Decoder) -> Self {
        if decoder.size() != decoder.size_out() {
            todo!()
        }

        // let (width, height) = decoder.size();
        // let time_base = decoder.time_base();

        // let builder = FilterGraphBuilder::new();
        // builder.add_filters(|ctx| {
        //     ctx.add_filter(BufferFilter::new("in", width, height, pix_fmt, time_base))
        // }).build()

        Self {
            decoder,
            // filter_graph: (),
        }
    }

    pub fn decode_raw_iter(&mut self) -> impl Iterator<Item = Result<RawFrame>> + '_ {
        std::iter::from_fn(move || Some(self.decode_raw()))
    }

    pub fn decode_raw(&mut self) -> Result<RawFrame> {
        let frame = self.decoder.decode_raw()?;

        let mut filter_graph = Self::build_filter_graph(&self.decoder, &frame)?;

        let mut source = filter_graph.source("in");
        let mut sink = filter_graph.sink("out");

        source.push_frame_raw(&frame);
        Ok(sink.pull_frame_raw())
    }

    fn build_filter_graph(decoder: &Decoder, frame: &RawFrame) -> Result<FilterGraph> {
        let time_base = decoder.time_base();

        let width = frame.width();
        let height = frame.height();
        let pixel_format = frame.format();

        let mut display_matrix: Option<Box<[i32]>> = None;

        let side_data = frame.side_data(ffmpeg::frame::side_data::Type::DisplayMatrix);
        if let Some(side_data) = side_data {
            let matrix: &[i32] = cast_slice(side_data.data());
            display_matrix = Some(matrix.into());
        }

        let stream = decoder.stream();
        if display_matrix.is_none() {
            if let Some(side_data) = stream.side_data().find(|side_data| {
                side_data.kind() == ffmpeg::codec::packet::side_data::Type::DisplayMatrix
            }) {
                let matrix: &[i32] = cast_slice(side_data.data());
                display_matrix = Some(matrix.into());
            }
        }

        let rotation = if let Some(display_matrix) = &display_matrix {
            unsafe { av_display_rotation_get(display_matrix.as_ptr()) }
        } else {
            0.0
        };

        Ok(FilterGraphBuilder::new()
            .add_filters(|ctx| {
                let source = ctx.add_filter(BufferFilter::new(
                    "in",
                    width,
                    height,
                    pixel_format,
                    time_base,
                ));

                let sink = ctx.add_filter(BufferSinkFilter::new("out"));

                if (rotation - 90.0).abs() < 1.0 {
                    let transpose = ctx.add_filter(TransposeFilter::new(
                        "transpose",
                        if display_matrix.unwrap()[3] > 0 {
                            TransposeDir::CClockFlip
                        } else {
                            TransposeDir::Clock
                        },
                    ));
                } else if (rotation - 180.0).abs() < 1.0 {
                    let display_matrix = display_matrix.unwrap();
                    if display_matrix[0] < 0 {
                        let hflip = ctx.add_filter(HFlipFilter::new("hflip"));
                    }
                    if display_matrix[4] < 0 {
                        let vflip = ctx.add_filter(VFlipFilter::new("vflip"));
                    }
                } else if (rotation - 270.0).abs() < 1.0 {
                    let transpose = ctx.add_filter(TransposeFilter::new(
                        "transpose",
                        if display_matrix.unwrap()[3] < 0 {
                            TransposeDir::ClockFlip
                        } else {
                            TransposeDir::CClock
                        },
                    ));
                } else if rotation.abs() > 1.0 {
                    let rotate = ctx.add_filter(
                        RotateFilter::new("rotate").with_angle(format!("{}*PI/180", rotation)),
                    );
                } else {
                    if display_matrix.is_some_and(|matrix| matrix[4] < 0) {
                        let vflip = ctx.add_filter(VFlipFilter::new("vflip"));
                    }
                }

                ctx.link(source.output(), sink.input());
            })
            .build())
    }
}
