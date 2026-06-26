extern crate ffmpeg_next as ffmpeg;

use bytemuck::cast_slice;
use ffmpeg_next::ffi::av_display_rotation_get;
use ffmpeg_next::format::Pixel;
use ffmpeg_next::Rational;

use crate::error::Error;
use crate::filter::{
    BufferFilter, BufferSinkFilter, FilterGraphBuilder, HFlipFilter, RotateFilter, ScaleFilter,
    TransposeDir, TransposeFilter, VFlipFilter,
};
use crate::frame::RawFrame;
use crate::{filter::FilterGraph, Decoder};

type Result<T> = std::result::Result<T, Error>;

pub struct VideoNormalizer {
    decoder: Decoder,
    filter_state: Option<(u32, u32, Pixel, Rational, FilterGraph)>,
}

impl VideoNormalizer {
    pub fn new(decoder: Decoder) -> Result<Self> {
        if decoder.size() != decoder.size_out() {
            todo!()
        }

        Ok(Self {
            decoder,
            filter_state: None,
        })
    }

    pub fn decode_raw_iter(&mut self) -> impl Iterator<Item = Result<RawFrame>> + '_ {
        std::iter::from_fn(move || Some(self.decode_raw()))
    }

    pub fn decode_raw(&mut self) -> Result<RawFrame> {
        let frame = self.decoder.decode_raw()?;

        let filter_rebuild =
            if let Some((last_w, last_h, last_format, last_sar, _)) = self.filter_state {
                last_w != frame.width()
                    || last_h != frame.height()
                    || last_format != frame.format()
                    || last_sar != frame.aspect_ratio()
            } else {
                true
            };

        if filter_rebuild {
            let filter_graph = Self::build_filter_graph(&self.decoder, &frame)?;
            self.filter_state = Some((
                frame.width(),
                frame.height(),
                frame.format(),
                frame.aspect_ratio(),
                filter_graph,
            ));
        }

        let (_, _, _, _, filter_graph) = self.filter_state.as_mut().unwrap();

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
        let sar = frame.aspect_ratio();

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
            Self::get_rotation(display_matrix)
        } else {
            0.0
        };

        FilterGraphBuilder::new()
            .add_filters(|ctx| {
                let source = ctx.add_filter(
                    BufferFilter::new("in", width, height, pixel_format, time_base)
                        .pixel_aspect(sar),
                );

                let sink = ctx.add_filter(BufferSinkFilter::new("out"));

                let mut last_input = sink.input();

                if (rotation - 90.0).abs() < 1.0 {
                    let transpose = ctx.add_filter(TransposeFilter::new(
                        "transpose",
                        if display_matrix.unwrap()[3] > 0 {
                            TransposeDir::CClockFlip
                        } else {
                            TransposeDir::Clock
                        },
                    ));

                    ctx.link(transpose.output(), last_input);
                    last_input = transpose.input();
                } else if (rotation - 180.0).abs() < 1.0 {
                    let display_matrix = display_matrix.unwrap();
                    if display_matrix[0] < 0 {
                        let hflip = ctx.add_filter(HFlipFilter::new("hflip"));
                        ctx.link(hflip.output(), last_input);
                        last_input = hflip.input();
                    }
                    if display_matrix[4] < 0 {
                        let vflip = ctx.add_filter(VFlipFilter::new("vflip"));
                        ctx.link(vflip.output(), last_input);
                        last_input = vflip.input();
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
                    ctx.link(transpose.output(), last_input);
                    last_input = transpose.input();
                } else if rotation.abs() > 1.0 {
                    let rotate = ctx.add_filter(
                        RotateFilter::new("rotate").with_angle(format!("{}*PI/180", rotation)),
                    );
                    ctx.link(rotate.output(), last_input);
                    last_input = rotate.input();
                } else {
                    if display_matrix.is_some_and(|matrix| matrix[4] < 0) {
                        let vflip = ctx.add_filter(VFlipFilter::new("vflip"));
                        ctx.link(vflip.output(), last_input);
                        last_input = vflip.input();
                    }
                }

                let scale = ctx.add_filter(ScaleFilter::new("scale", "iw*sar", "ih"));
                ctx.link(scale.output(), last_input);
                ctx.link(source.output(), scale.input());
            })
            .build()
    }

    fn get_rotation(display_matrix: &[i32]) -> f64 {
        let theta = -(unsafe { av_display_rotation_get(display_matrix.as_ptr()) }.round());

        let theta = theta - 360.0 * (theta / 360.0 + 0.9 / 360.0).floor();

        println!("rotation: {}", theta);

        theta
    }
}
